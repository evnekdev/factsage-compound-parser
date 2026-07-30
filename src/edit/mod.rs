//! Controlled edits over the lossless flat raw CDB stream.
//!
//! [`DatabaseEditor`] owns one authoritative [`crate::RawDatabase`] plus an
//! optional lightweight [`crate::domain::DomainIndex`]. Non-structural setters
//! change only documented raw fields and retain a current index. Structural raw
//! edits invalidate that index; the next [`DatabaseEditor::view`] rebuilds it
//! deterministically before borrowing semantic views.

mod error;

use std::io::{Read, Write};
use std::path::Path;

use crate::domain::{Database, DatabaseView, DomainError, DomainIndex};
use crate::raw::{RawChunk, RawEditError, SerializeError};
use crate::thermo::{EnergyUnit, PhaseKind};
use crate::{ParseError, RawDatabase};

/// Re-export of controlled-edit errors.
pub use error::EditError;

/// A raw-authoritative CDB editor with lazily rebuilt semantic indexes.
///
/// The raw stream is the only mutable source of truth. The optional index stores
/// only relationships and diagnostics; it never owns a second copy of chunks.
/// Semantic views borrow the editor, so Rust prevents them from surviving a
/// mutable edit. Structural edits invalidate the index; supported field setters
/// leave it valid because they do not change chunk order, IDs, or links.
#[derive(Debug)]
pub struct DatabaseEditor {
    raw: RawDatabase,
    index: Option<DomainIndex>,
}

impl Clone for DatabaseEditor {
    /// Clones the authoritative raw stream but intentionally drops the cached index.
    ///
    /// The clone has a distinct raw-stream identity, so rebuilding on first view
    /// access avoids using an index associated with the original allocation.
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            index: None,
        }
    }
}

impl PartialEq for DatabaseEditor {
    /// Compares authoritative raw chunks and intentionally ignores lazy cache state.
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl DatabaseEditor {
    /// Starts editing an already parsed flat raw database without building an index.
    pub const fn from_raw(raw: RawDatabase) -> Self {
        Self { raw, index: None }
    }

    /// Parses bytes into an editable lossless raw database.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        RawDatabase::from_bytes(bytes).map(Self::from_raw)
    }

    /// Streams records into an editable lossless raw database.
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, ParseError> {
        RawDatabase::from_reader(reader).map(Self::from_raw)
    }

    /// Opens a path into an editable lossless raw database using streaming I/O.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        RawDatabase::from_path(path).map(Self::from_raw)
    }

    /// Returns the sole authoritative raw stream after any edits.
    pub const fn raw(&self) -> &RawDatabase {
        &self.raw
    }

    /// Returns mutable raw chunks and invalidates the cached semantic index.
    ///
    /// This low-level API permits semantically invalid physical streams. Call
    /// [`Self::view`] afterward to rebuild and validate semantic grouping.
    pub fn raw_mut(&mut self) -> &mut RawDatabase {
        self.index = None;
        &mut self.raw
    }

    /// Returns whether a cached semantic index currently matches the raw stream.
    pub fn has_current_index(&self) -> bool {
        self.index
            .as_ref()
            .is_some_and(|index| index.is_current_for(&self.raw))
    }

    /// Rebuilds the semantic index from the authoritative raw stream.
    ///
    /// A grouping error leaves the raw stream unchanged and leaves no cached
    /// index, allowing callers to repair a low-level structural edit.
    pub fn rebuild_index(&mut self) -> Result<(), DomainError> {
        self.index = None;
        let index = DomainIndex::build(&self.raw)?;
        self.index = Some(index);
        Ok(())
    }

    /// Lazily rebuilds the semantic index and returns a borrowed read-only view.
    pub fn view(&mut self) -> Result<DatabaseView<'_>, DomainError> {
        if !self.has_current_index() {
            self.rebuild_index()?;
        }
        let Some(index) = self.index.as_ref() else {
            return Err(DomainError::EmptyRawDatabase);
        };
        DatabaseView::new(&self.raw, index)
    }

    /// Backward-compatible name for [`Self::view`].
    ///
    /// The return type is now a borrowed [`DatabaseView`] rather than an owned
    /// duplicate semantic model. Migrate by iterating `editor.view()?.compounds()`.
    pub fn domain(&mut self) -> Result<DatabaseView<'_>, DomainError> {
        self.view()
    }

    /// Consumes the editor and returns the authoritative raw stream.
    pub fn into_raw(self) -> RawDatabase {
        self.raw
    }

    /// Consumes the editor and builds an owned raw-plus-index database handle.
    pub fn into_domain(self) -> Result<Database, DomainError> {
        Database::from_raw(self.raw)
    }

    /// Serializes the current authoritative raw stream without forcing an index rebuild.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerializeError> {
        self.raw.to_bytes()
    }

    /// Streams the current authoritative raw stream to a byte sink.
    pub fn write_to<W: Write>(&self, writer: W) -> Result<(), SerializeError> {
        self.raw.write_to(writer)
    }

    /// Creates or replaces a caller-selected file with the edited raw stream.
    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<(), SerializeError> {
        self.raw.write_to_path(path)
    }

    /// Inserts a physical raw chunk and invalidates the semantic index.
    ///
    /// The resulting stream may be semantically invalid until [`Self::view`]
    /// rebuilds and validates the documented grouping state machine.
    pub fn insert_chunk(&mut self, index: usize, chunk: RawChunk) -> Result<(), RawEditError> {
        self.raw.insert_chunk(index, chunk)?;
        self.index = None;
        Ok(())
    }

    /// Appends a physical raw chunk and invalidates the semantic index.
    pub fn push_chunk(&mut self, chunk: RawChunk) {
        self.raw.push_chunk(chunk);
        self.index = None;
    }

    /// Removes a physical raw chunk and invalidates the semantic index.
    pub fn remove_chunk(&mut self, index: usize) -> Result<RawChunk, RawEditError> {
        let chunk = self.raw.remove_chunk(index)?;
        self.index = None;
        Ok(chunk)
    }

    /// Sets a compound name as strict ASCII in its 40-byte fixed-width field.
    ///
    /// This non-structural edit NUL-pads the complete field deterministically and
    /// changes no bytes outside it, so a cached index remains valid.
    pub fn set_compound_name(
        &mut self,
        compound_index: usize,
        name: &str,
    ) -> Result<(), EditError> {
        let compound = self.compound_mut(compound_index)?;
        set_fixed_ascii("compound_name", &mut compound.compound_name, name)
    }

    /// Sets a phase name as strict ASCII in its 40-byte fixed-width field.
    ///
    /// This non-structural edit NUL-pads the complete field deterministically and
    /// changes no bytes outside it, so a cached index remains valid.
    pub fn set_phase_name(
        &mut self,
        compound_index: usize,
        phase_index: usize,
        name: &str,
    ) -> Result<(), EditError> {
        let chunk_index = self.phase_chunk_index(compound_index, phase_index)?;
        match self.raw.chunk_mut_without_invalidation(chunk_index) {
            Some(RawChunk::PhaseOrdinary(chunk)) => {
                set_fixed_ascii("phase_name", &mut chunk.physical.phase_name, name)
            }
            Some(RawChunk::PhaseTransition(chunk)) => {
                set_fixed_ascii("phase_name", &mut chunk.physical.phase_name, name)
            }
            Some(_) | None => Err(EditError::InconsistentRawStream { chunk_index }),
        }
    }

    /// Sets an ordinary phase enthalpy from finite joules per mole.
    ///
    /// The inverse conversion uses the owning compound's raw energy unit. Stored
    /// CP anchor values are deliberately not changed because their reference
    /// convention remains unverified. This non-structural edit retains the index.
    pub fn set_ordinary_phase_enthalpy_298_j_per_mol(
        &mut self,
        compound_index: usize,
        phase_index: usize,
        enthalpy_j_per_mol: f64,
    ) -> Result<(), EditError> {
        let raw_value = self
            .energy_unit(compound_index)?
            .from_joules(enthalpy_j_per_mol)?;
        let chunk_index = self.phase_chunk_index(compound_index, phase_index)?;
        match self.raw.chunk_mut_without_invalidation(chunk_index) {
            Some(RawChunk::PhaseOrdinary(chunk)) => {
                chunk.enthalpy = raw_value;
                Ok(())
            }
            Some(RawChunk::PhaseTransition(_)) => Err(wrong_phase_type(
                compound_index,
                phase_index,
                PhaseKind::Ordinary,
                PhaseKind::Transition,
            )),
            Some(_) | None => Err(EditError::InconsistentRawStream { chunk_index }),
        }
    }

    /// Sets an ordinary phase entropy from finite joules per mole kelvin.
    ///
    /// The inverse conversion uses the owning compound's raw energy unit. This
    /// non-structural edit does not propagate to CP records and retains the index.
    pub fn set_ordinary_phase_entropy_298_j_per_mol_k(
        &mut self,
        compound_index: usize,
        phase_index: usize,
        entropy_j_per_mol_k: f64,
    ) -> Result<(), EditError> {
        let raw_value = self
            .energy_unit(compound_index)?
            .from_joules(entropy_j_per_mol_k)?;
        let chunk_index = self.phase_chunk_index(compound_index, phase_index)?;
        match self.raw.chunk_mut_without_invalidation(chunk_index) {
            Some(RawChunk::PhaseOrdinary(chunk)) => {
                chunk.entropy = raw_value;
                Ok(())
            }
            Some(RawChunk::PhaseTransition(_)) => Err(wrong_phase_type(
                compound_index,
                phase_index,
                PhaseKind::Ordinary,
                PhaseKind::Transition,
            )),
            Some(_) | None => Err(EditError::InconsistentRawStream { chunk_index }),
        }
    }

    /// Sets a transition phase enthalpy from finite joules per mole.
    ///
    /// The inverse conversion correctly leaves joule-based records unchanged and
    /// divides only calorie-based records by 4.184. This non-structural edit
    /// retains the index.
    pub fn set_transition_phase_enthalpy_j_per_mol(
        &mut self,
        compound_index: usize,
        phase_index: usize,
        enthalpy_j_per_mol: f64,
    ) -> Result<(), EditError> {
        let raw_value = self
            .energy_unit(compound_index)?
            .from_joules(enthalpy_j_per_mol)?;
        let chunk_index = self.phase_chunk_index(compound_index, phase_index)?;
        match self.raw.chunk_mut_without_invalidation(chunk_index) {
            Some(RawChunk::PhaseTransition(chunk)) => {
                chunk.transition_enthalpy = raw_value;
                Ok(())
            }
            Some(RawChunk::PhaseOrdinary(_)) => Err(wrong_phase_type(
                compound_index,
                phase_index,
                PhaseKind::Transition,
                PhaseKind::Ordinary,
            )),
            Some(_) | None => Err(EditError::InconsistentRawStream { chunk_index }),
        }
    }

    /// Sets a finite transition temperature in kelvin without unit conversion.
    ///
    /// This is a non-structural edit and retains the cached semantic index.
    pub fn set_transition_phase_temperature_k(
        &mut self,
        compound_index: usize,
        phase_index: usize,
        temperature_k: f64,
    ) -> Result<(), EditError> {
        ensure_finite("transition_temperature", temperature_k)?;
        let chunk_index = self.phase_chunk_index(compound_index, phase_index)?;
        match self.raw.chunk_mut_without_invalidation(chunk_index) {
            Some(RawChunk::PhaseTransition(chunk)) => {
                chunk.transition_temperature = temperature_k;
                Ok(())
            }
            Some(RawChunk::PhaseOrdinary(_)) => Err(wrong_phase_type(
                compound_index,
                phase_index,
                PhaseKind::Transition,
                PhaseKind::Ordinary,
            )),
            Some(_) | None => Err(EditError::InconsistentRawStream { chunk_index }),
        }
    }

    /// Replaces all seven finite real stoichiometric coefficients.
    ///
    /// This non-structural edit updates only the seven stored `f64` fields and
    /// retains the cached semantic index.
    pub fn set_real_stoichiometric_coefficients(
        &mut self,
        compound_index: usize,
        coefficients: [f64; 7],
    ) -> Result<(), EditError> {
        for value in coefficients {
            ensure_finite("real_stoichiometric_coefficients", value)?;
        }
        self.compound_mut(compound_index)?
            .real_stoichiometric_coefficients = coefficients;
        Ok(())
    }

    fn compound_mut(
        &mut self,
        compound_index: usize,
    ) -> Result<&mut crate::RawCompoundChunk, EditError> {
        let chunk_index = self.compound_chunk_index(compound_index)?;
        match self.raw.chunk_mut_without_invalidation(chunk_index) {
            Some(RawChunk::Compound(chunk)) => Ok(chunk),
            Some(_) | None => Err(EditError::InconsistentRawStream { chunk_index }),
        }
    }

    fn energy_unit(&self, compound_index: usize) -> Result<EnergyUnit, EditError> {
        let chunk_index = self.compound_chunk_index(compound_index)?;
        match self.raw.chunks().get(chunk_index) {
            Some(RawChunk::Compound(chunk)) => Ok(EnergyUnit::from_raw(chunk.unit_energy)),
            Some(_) | None => Err(EditError::InconsistentRawStream { chunk_index }),
        }
    }

    fn compound_chunk_index(&self, compound_index: usize) -> Result<usize, EditError> {
        self.raw
            .chunks()
            .iter()
            .enumerate()
            .filter(|(_, chunk)| matches!(chunk, RawChunk::Compound(_)))
            .nth(compound_index)
            .map(|(chunk_index, _)| chunk_index)
            .ok_or(EditError::CompoundNotFound { compound_index })
    }

    fn phase_chunk_index(
        &self,
        compound_index: usize,
        phase_index: usize,
    ) -> Result<usize, EditError> {
        let compound_chunk_index = self.compound_chunk_index(compound_index)?;
        let chunks = self.raw.chunks();
        let Some(after_compound) = chunks.get(compound_chunk_index.saturating_add(1)..) else {
            return Err(EditError::InconsistentRawStream {
                chunk_index: compound_chunk_index,
            });
        };
        let next_compound_chunk_index = after_compound
            .iter()
            .position(|chunk| matches!(chunk, RawChunk::Compound(_)))
            .map_or(chunks.len(), |relative_index| {
                compound_chunk_index + 1 + relative_index
            });
        let Some(group) = chunks.get(compound_chunk_index + 1..next_compound_chunk_index) else {
            return Err(EditError::InconsistentRawStream {
                chunk_index: compound_chunk_index,
            });
        };
        group
            .iter()
            .enumerate()
            .filter(|(_, chunk)| {
                matches!(
                    chunk,
                    RawChunk::PhaseOrdinary(_) | RawChunk::PhaseTransition(_)
                )
            })
            .nth(phase_index)
            .map(|(relative_index, _)| compound_chunk_index + 1 + relative_index)
            .ok_or(EditError::PhaseNotFound {
                compound_index,
                phase_index,
            })
    }
}

fn wrong_phase_type(
    compound_index: usize,
    phase_index: usize,
    expected: PhaseKind,
    actual: PhaseKind,
) -> EditError {
    EditError::WrongPhaseType {
        compound_index,
        phase_index,
        expected,
        actual,
    }
}

fn ensure_finite(field: &'static str, value: f64) -> Result<(), EditError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(EditError::NonFiniteValue { field, value })
    }
}

fn set_fixed_ascii(
    field: &'static str,
    destination: &mut [u8],
    value: &str,
) -> Result<(), EditError> {
    if let Some((byte_index, _)) = value
        .char_indices()
        .find(|(_, character)| !character.is_ascii())
    {
        return Err(EditError::NonAsciiText { field, byte_index });
    }
    if value.as_bytes().contains(&0) {
        return Err(EditError::EmbeddedNul { field });
    }
    if value.len() > destination.len() {
        return Err(EditError::TextTooLong {
            field,
            maximum: destination.len(),
            actual: value.len(),
        });
    }

    destination.fill(0);
    destination[..value.len()].copy_from_slice(value.as_bytes());
    Ok(())
}
