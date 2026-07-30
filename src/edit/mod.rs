//! Controlled edits over the lossless flat raw CDB stream.
//!
//! [`DatabaseEditor`] keeps [`crate::RawDatabase`] as its sole mutable source
//! of truth. Build a fresh [`crate::domain::Database`] view after edits instead
//! of maintaining a second mutable semantic copy.

mod error;

use std::io::{Read, Write};
use std::path::Path;

use crate::domain::{Database, DomainError};
use crate::raw::{RawChunk, SerializeError};
use crate::thermo::{EnergyUnit, PhaseKind};
use crate::{ParseError, RawDatabase};

pub use error::EditError;

/// A raw-authoritative editor for established CDB fields.
///
/// The editor changes only selected byte-backed fields in [`RawDatabase`]. Its
/// [`Self::domain`] method rebuilds a semantic view from a clone of that raw
/// stream, so domain links can never silently diverge from serialized output.
#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseEditor {
    raw: RawDatabase,
}

impl DatabaseEditor {
    /// Starts editing an already parsed flat raw database.
    pub const fn from_raw(raw: RawDatabase) -> Self {
        Self { raw }
    }

    /// Parses bytes into an editable lossless raw database.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        RawDatabase::from_bytes(bytes).map(Self::from_raw)
    }

    /// Reads and parses an editable lossless raw database.
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, ParseError> {
        RawDatabase::from_reader(reader).map(Self::from_raw)
    }

    /// Opens and parses an editable lossless raw database.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        RawDatabase::from_path(path).map(Self::from_raw)
    }

    /// Returns the authoritative raw stream after any edits.
    pub const fn raw(&self) -> &RawDatabase {
        &self.raw
    }

    /// Consumes the editor and returns the authoritative raw stream.
    pub fn into_raw(self) -> RawDatabase {
        self.raw
    }

    /// Rebuilds a semantic view from the current authoritative raw stream.
    pub fn domain(&self) -> Result<Database, DomainError> {
        Database::from_raw(self.raw.clone())
    }

    /// Consumes the editor and builds a semantic view from its raw stream.
    pub fn into_domain(self) -> Result<Database, DomainError> {
        Database::from_raw(self.raw)
    }

    /// Serializes the current authoritative raw stream.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerializeError> {
        self.raw.to_bytes()
    }

    /// Writes the current authoritative raw stream to a byte sink.
    pub fn write_to<W: Write>(&self, writer: W) -> Result<(), SerializeError> {
        self.raw.write_to(writer)
    }

    /// Creates or replaces a caller-selected file with the edited raw stream.
    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<(), SerializeError> {
        self.raw.write_to_path(path)
    }

    /// Sets a compound name as strict ASCII in its 40-byte fixed-width field.
    ///
    /// The complete field is deterministically NUL-padded after the supplied
    /// text; no bytes outside that field are changed.
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
    /// The complete field is deterministically NUL-padded after the supplied
    /// text; no bytes outside that field are changed.
    pub fn set_phase_name(
        &mut self,
        compound_index: usize,
        phase_index: usize,
        name: &str,
    ) -> Result<(), EditError> {
        let chunk_index = self.phase_chunk_index(compound_index, phase_index)?;
        match self.raw.chunks.get_mut(chunk_index) {
            Some(RawChunk::PhaseOrdinary(chunk)) => {
                set_fixed_ascii("phase_name", &mut chunk.physical.phase_name, name)
            }
            Some(RawChunk::PhaseTransition(chunk)) => {
                set_fixed_ascii("phase_name", &mut chunk.physical.phase_name, name)
            }
            Some(_) | None => Err(EditError::InconsistentRawStream { chunk_index }),
        }
    }

    /// Sets an ordinary phase enthalpy from joules per mole.
    ///
    /// The value is converted with the owning compound's established energy
    /// unit. Stored CP anchors are deliberately not changed.
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
        match self.raw.chunks.get_mut(chunk_index) {
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

    /// Sets an ordinary phase entropy from joules per mole kelvin.
    ///
    /// The value is converted with the owning compound's established energy
    /// unit. Stored CP anchors are deliberately not changed.
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
        match self.raw.chunks.get_mut(chunk_index) {
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

    /// Sets a transition phase enthalpy from joules per mole.
    ///
    /// The inverse conversion is applied only for a calorie-based compound.
    /// This intentionally corrects the Python setter's unconditional division
    /// by 4.184 for joule-based compounds.
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
        match self.raw.chunks.get_mut(chunk_index) {
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
    pub fn set_transition_phase_temperature_k(
        &mut self,
        compound_index: usize,
        phase_index: usize,
        temperature_k: f64,
    ) -> Result<(), EditError> {
        ensure_finite("transition_temperature", temperature_k)?;
        let chunk_index = self.phase_chunk_index(compound_index, phase_index)?;
        match self.raw.chunks.get_mut(chunk_index) {
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

    /// Replaces all seven real stoichiometric coefficients.
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
        match self.raw.chunks.get_mut(chunk_index) {
            Some(RawChunk::Compound(chunk)) => Ok(chunk),
            Some(_) | None => Err(EditError::InconsistentRawStream { chunk_index }),
        }
    }

    fn energy_unit(&self, compound_index: usize) -> Result<EnergyUnit, EditError> {
        let chunk_index = self.compound_chunk_index(compound_index)?;
        match self.raw.chunks.get(chunk_index) {
            Some(RawChunk::Compound(chunk)) => Ok(EnergyUnit::from_raw(chunk.unit_energy)),
            Some(_) | None => Err(EditError::InconsistentRawStream { chunk_index }),
        }
    }

    fn compound_chunk_index(&self, compound_index: usize) -> Result<usize, EditError> {
        self.raw
            .chunks
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
        let next_compound_chunk_index = self.raw.chunks[compound_chunk_index + 1..]
            .iter()
            .position(|chunk| matches!(chunk, RawChunk::Compound(_)))
            .map_or(self.raw.chunks.len(), |relative_index| {
                compound_chunk_index + 1 + relative_index
            });

        self.raw.chunks[compound_chunk_index + 1..next_compound_chunk_index]
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
