use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::raw::{RawChunk, RawDatabaseHeaderChunk};
use crate::{CHUNK_SIZE, RawDatabase};

use super::compound::CompoundView;
use super::error::{DatabaseError, Diagnostic, DiagnosticKind, DomainError, ExpectedCategory};
use super::phase::{PhaseState, RawPhase};
use super::range::OrphanReason;
use super::text::decode_ascii;

/// A lightweight semantic index over one immutable-generation raw CDB stream.
///
/// The index stores physical chunk indexes, range links, and diagnostics only;
/// it deliberately does not clone complete raw chunks. It is tied to the exact
/// [`RawDatabase`] identity and structural generation that produced it. Build a
/// new index after any call to [`RawDatabase::chunks_mut`],
/// [`RawDatabase::insert_chunk`], [`RawDatabase::push_chunk`], or
/// [`RawDatabase::remove_chunk`].
#[derive(Debug, Clone, PartialEq)]
pub struct DomainIndex {
    compounds: Vec<CompoundIndex>,
    diagnostics: Vec<Diagnostic>,
    raw_identity: u64,
    raw_generation: u64,
}

/// An owned read-only database handle containing raw chunks plus their index.
///
/// This convenience owner keeps a single raw stream and a lightweight index. Use
/// [`Self::view`] to obtain borrowed semantic access, or [`Self::into_raw`] to
/// move the authoritative stream into [`crate::DatabaseEditor`].
#[derive(Debug)]
pub struct Database {
    raw: RawDatabase,
    index: DomainIndex,
}

/// Borrowed read-only semantic access over a raw CDB stream and matching index.
///
/// Creating a view validates that the index belongs to the supplied raw stream.
/// Views neither allocate nor clone raw records during normal traversal. Rust's
/// borrowing rules prevent a view from surviving a mutable raw or editor borrow.
#[derive(Debug, Clone, Copy)]
pub struct DatabaseView<'a> {
    raw: &'a RawDatabase,
    index: &'a DomainIndex,
    header: &'a RawDatabaseHeaderChunk,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CompoundIndex {
    pub(crate) compound_chunk: usize,
    pub(crate) phase_indexes: Vec<PhaseIndex>,
    pub(crate) comment_chunks: Vec<usize>,
    pub(crate) orphan_ranges: Vec<OrphanRangeIndex>,
    pub(crate) unknown_chunks: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PhaseIndex {
    pub(crate) phase_chunk: usize,
    pub(crate) heat_capacity_chunks: Vec<usize>,
    pub(crate) kappa_chunks: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OrphanRangeIndex {
    pub(crate) chunk_index: usize,
    pub(crate) byte_offset: usize,
    pub(crate) phase_id_raw: i32,
    pub(crate) reason: OrphanReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupStage {
    Phases,
    Ranges,
    Comments,
}

impl DomainIndex {
    /// Builds a semantic index for the current generation of `raw`.
    ///
    /// Invalid known-chunk ordering is fatal and returns [`DomainError`].
    /// Duplicate phase IDs, orphan or ambiguous ranges, questionable IDs,
    /// invalid ASCII, invalid range bounds, and unknown chunks are retained as
    /// typed [`Diagnostic`] values without discarding their raw records.
    pub fn build(raw: &RawDatabase) -> Result<Self, DomainError> {
        let chunks = raw.chunks();
        let Some(first_chunk) = chunks.first() else {
            return Err(DomainError::EmptyRawDatabase);
        };
        let RawChunk::DatabaseHeader(header) = first_chunk else {
            return Err(DomainError::MissingDatabaseHeader {
                found: Some(first_chunk.id()),
            });
        };

        let mut diagnostics = Vec::new();
        validate_ascii_field(
            &header.comment,
            "database_header.comment",
            0,
            None,
            &mut diagnostics,
        );

        let mut compounds = Vec::new();
        let mut position = 1_usize;
        while position < chunks.len() {
            let chunk = &chunks[position];
            if chunk.id() != 1 {
                return Err(ordering_error(
                    position,
                    chunk.id(),
                    ExpectedCategory::Compound,
                    None,
                ));
            }
            let RawChunk::Compound(compound_raw) = chunk else {
                return Err(ordering_error(
                    position,
                    chunk.id(),
                    ExpectedCategory::Compound,
                    None,
                ));
            };

            let compound_index = compounds.len();
            validate_compound_text(compound_raw, position, compound_index, &mut diagnostics);
            let mut compound = CompoundIndex {
                compound_chunk: position,
                phase_indexes: Vec::new(),
                comment_chunks: Vec::new(),
                orphan_ranges: Vec::new(),
                unknown_chunks: Vec::new(),
            };
            let mut phase_indices: HashMap<i32, Vec<usize>> = HashMap::new();
            let mut stage = GroupStage::Phases;
            position += 1;

            while position < chunks.len() {
                let next = &chunks[position];
                let next_id = next.id();
                if next_id == 1 {
                    break;
                }

                if !is_known_id(next_id) {
                    diagnostics.push(Diagnostic {
                        chunk_index: position,
                        byte_offset: position.saturating_mul(CHUNK_SIZE),
                        compound_index: Some(compound_index),
                        kind: DiagnosticKind::UnknownChunk { id: next_id },
                    });
                    compound.unknown_chunks.push(position);
                    position += 1;
                    continue;
                }

                match stage {
                    GroupStage::Phases if is_phase_id(next_id) => {
                        let raw_phase = match next {
                            RawChunk::PhaseOrdinary(chunk) => RawPhase::Ordinary(chunk),
                            RawChunk::PhaseTransition(chunk) => RawPhase::Transition(chunk),
                            _ => {
                                return Err(ordering_error(
                                    position,
                                    next_id,
                                    ExpectedCategory::PhaseRangeCommentOrCompound,
                                    Some(compound_index),
                                ));
                            }
                        };
                        validate_phase(raw_phase, position, compound_index, &mut diagnostics);
                        let phase_index = compound.phase_indexes.len();
                        let phase_id_raw = raw_phase.phase_id_raw();
                        let candidates = phase_indices.entry(phase_id_raw).or_default();
                        candidates.push(phase_index);
                        if candidates.len() > 1 {
                            diagnostics.push(Diagnostic {
                                chunk_index: position,
                                byte_offset: position.saturating_mul(CHUNK_SIZE),
                                compound_index: Some(compound_index),
                                kind: DiagnosticKind::DuplicatePhaseId { phase_id_raw },
                            });
                        }
                        compound.phase_indexes.push(PhaseIndex {
                            phase_chunk: position,
                            heat_capacity_chunks: Vec::new(),
                            kappa_chunks: Vec::new(),
                        });
                        position += 1;
                    }
                    GroupStage::Phases if is_range_id(next_id) => stage = GroupStage::Ranges,
                    GroupStage::Phases if next_id == 10 => stage = GroupStage::Comments,
                    GroupStage::Phases => {
                        return Err(ordering_error(
                            position,
                            next_id,
                            ExpectedCategory::PhaseRangeCommentOrCompound,
                            Some(compound_index),
                        ));
                    }
                    GroupStage::Ranges if is_range_id(next_id) => {
                        validate_range(next, position, compound_index, &mut diagnostics);
                        attach_range(
                            &mut compound,
                            &phase_indices,
                            next,
                            position,
                            compound_index,
                            &mut diagnostics,
                        );
                        position += 1;
                    }
                    GroupStage::Ranges if next_id == 10 => stage = GroupStage::Comments,
                    GroupStage::Ranges => {
                        return Err(ordering_error(
                            position,
                            next_id,
                            ExpectedCategory::RangeCommentOrCompound,
                            Some(compound_index),
                        ));
                    }
                    GroupStage::Comments if next_id == 10 => {
                        compound.comment_chunks.push(position);
                        position += 1;
                    }
                    GroupStage::Comments => {
                        return Err(ordering_error(
                            position,
                            next_id,
                            ExpectedCategory::CommentOrCompound,
                            Some(compound_index),
                        ));
                    }
                }
            }

            compounds.push(compound);
        }

        let (raw_identity, raw_generation) = raw.index_token();
        Ok(Self {
            compounds,
            diagnostics,
            raw_identity,
            raw_generation,
        })
    }

    /// Returns the number of compound groups in stream order.
    pub fn compound_count(&self) -> usize {
        self.compounds.len()
    }

    /// Returns non-fatal diagnostics in physical stream order.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns whether this index was built for the current structural generation of `raw`.
    pub fn is_current_for(&self, raw: &RawDatabase) -> bool {
        (self.raw_identity, self.raw_generation) == raw.index_token()
    }

    pub(crate) fn ensure_current(&self, raw: &RawDatabase) -> Result<(), DomainError> {
        let (raw_identity, raw_generation) = raw.index_token();
        if (self.raw_identity, self.raw_generation) == (raw_identity, raw_generation) {
            Ok(())
        } else {
            Err(DomainError::StaleIndex {
                index_identity: self.raw_identity,
                index_generation: self.raw_generation,
                raw_identity,
                raw_generation,
            })
        }
    }
}

impl Database {
    /// Parses raw CDB bytes and builds a lightweight semantic index.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DatabaseError> {
        Self::from_raw(RawDatabase::from_bytes(bytes)?).map_err(DatabaseError::Domain)
    }

    /// Streams CDB records from a reader and builds a semantic index.
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, DatabaseError> {
        Self::from_raw(RawDatabase::from_reader(reader)?).map_err(DatabaseError::Domain)
    }

    /// Opens a CDB path with streaming I/O and builds a semantic index.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, DatabaseError> {
        Self::from_reader(File::open(path).map_err(crate::error::ParseError::from)?)
    }

    /// Takes ownership of a raw stream and builds its lightweight semantic index.
    pub fn from_raw(raw: RawDatabase) -> Result<Self, DomainError> {
        let index = DomainIndex::build(&raw)?;
        Ok(Self { raw, index })
    }

    /// Returns the authoritative raw stream without allowing mutation.
    pub const fn raw(&self) -> &RawDatabase {
        &self.raw
    }

    /// Returns the matching lightweight semantic index.
    pub const fn index(&self) -> &DomainIndex {
        &self.index
    }

    /// Creates a borrowed semantic view over this owned raw stream.
    pub fn view(&self) -> Result<DatabaseView<'_>, DomainError> {
        DatabaseView::new(&self.raw, &self.index)
    }

    /// Moves the single authoritative raw stream out of this owner.
    ///
    /// The index is discarded because it borrows no data and can be rebuilt for
    /// any subsequent owner or editor.
    pub fn into_raw(self) -> RawDatabase {
        self.raw
    }
}

impl TryFrom<RawDatabase> for Database {
    type Error = DomainError;

    /// Builds an owned semantic database from a lossless raw stream.
    fn try_from(raw: RawDatabase) -> Result<Self, Self::Error> {
        Self::from_raw(raw)
    }
}

impl<'a> DatabaseView<'a> {
    /// Creates a borrowed semantic view after checking the raw stream identity and generation.
    pub fn new(raw: &'a RawDatabase, index: &'a DomainIndex) -> Result<Self, DomainError> {
        index.ensure_current(raw)?;
        let Some(RawChunk::DatabaseHeader(header)) = raw.chunks().first() else {
            return Err(DomainError::MissingDatabaseHeader {
                found: raw.chunks().first().map(RawChunk::id),
            });
        };
        Ok(Self { raw, index, header })
    }

    /// Returns the required first database-header chunk.
    pub const fn header(self) -> &'a RawDatabaseHeaderChunk {
        self.header
    }

    /// Returns the authoritative raw stream borrowed by this view.
    pub const fn raw(self) -> &'a RawDatabase {
        self.raw
    }

    /// Returns the lightweight index borrowed by this view.
    pub const fn index(self) -> &'a DomainIndex {
        self.index
    }

    /// Returns non-fatal semantic diagnostics without allocating or cloning them.
    pub fn diagnostics(self) -> &'a [Diagnostic] {
        self.index.diagnostics()
    }

    /// Iterates compounds in original stream order without cloning raw records.
    pub fn compounds(self) -> impl Iterator<Item = CompoundView<'a>> + 'a {
        self.index.compounds.iter().filter_map(move |index| {
            let Some(RawChunk::Compound(raw)) = self.raw.chunks().get(index.compound_chunk) else {
                return None;
            };
            Some(CompoundView::new(raw, index, self.raw.chunks()))
        })
    }

    /// Returns the number of compounds indexed in stream order.
    pub fn compound_count(self) -> usize {
        self.index.compound_count()
    }

    /// Finds the first compound whose decoded formula exactly matches `formula`.
    pub fn find_compound_by_formula(self, formula: &str) -> Option<CompoundView<'a>> {
        self.compounds()
            .find(|compound| compound.formula().is_ok_and(|value| value == formula))
    }

    /// Finds the first compound whose decoded name starts with `prefix`.
    pub fn find_compound_by_name(self, prefix: &str) -> Option<CompoundView<'a>> {
        self.compounds()
            .find(|compound| compound.name().is_ok_and(|value| value.starts_with(prefix)))
    }
}

fn is_known_id(id: u8) -> bool {
    (1..=11).contains(&id)
}

fn is_phase_id(id: u8) -> bool {
    matches!(id, 7 | 8)
}

fn is_range_id(id: u8) -> bool {
    (2..=6).contains(&id) || id == 11
}

fn ordering_error(
    chunk_index: usize,
    chunk_id: u8,
    expected: ExpectedCategory,
    compound_index: Option<usize>,
) -> DomainError {
    DomainError::Ordering {
        chunk_index,
        byte_offset: chunk_index.saturating_mul(CHUNK_SIZE),
        chunk_id,
        expected,
        compound_index,
    }
}

fn validate_compound_text(
    compound: &crate::RawCompoundChunk,
    chunk_index: usize,
    compound_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    validate_ascii_field(
        &compound.compound_name,
        "compound_name",
        chunk_index,
        Some(compound_index),
        diagnostics,
    );
    validate_ascii_field(
        &compound.formula_name,
        "formula_name",
        chunk_index,
        Some(compound_index),
        diagnostics,
    );
}

fn validate_phase(
    phase: RawPhase<'_>,
    chunk_index: usize,
    compound_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let bytes = match phase {
        RawPhase::Ordinary(chunk) => &chunk.physical.phase_name,
        RawPhase::Transition(chunk) => &chunk.physical.phase_name,
    };
    validate_ascii_field(
        bytes,
        "phase_name",
        chunk_index,
        Some(compound_index),
        diagnostics,
    );
    let state = PhaseState::from_raw_id(phase.phase_id_raw());
    let index = phase_index(phase.phase_id_raw(), state);
    if index <= 0 {
        diagnostics.push(Diagnostic {
            chunk_index,
            byte_offset: chunk_index.saturating_mul(CHUNK_SIZE),
            compound_index: Some(compound_index),
            kind: DiagnosticKind::SuspiciousPhaseId {
                phase_id_raw: phase.phase_id_raw(),
                state,
                index,
            },
        });
    }
}

fn phase_index(phase_id_raw: i32, state: PhaseState) -> i32 {
    match state {
        PhaseState::Aqueous => phase_id_raw - 990,
        PhaseState::Gas => phase_id_raw - 900,
        PhaseState::Liquid => phase_id_raw - 800,
        PhaseState::Solid => phase_id_raw - 100,
    }
}

fn validate_ascii_field(
    bytes: &[u8],
    field: &'static str,
    chunk_index: usize,
    compound_index: Option<usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if decode_ascii(bytes, field).is_err() {
        diagnostics.push(Diagnostic {
            chunk_index,
            byte_offset: chunk_index.saturating_mul(CHUNK_SIZE),
            compound_index,
            kind: DiagnosticKind::InvalidAsciiText {
                field: field.to_owned(),
            },
        });
    }
}

fn range_bounds(chunk: &RawChunk) -> Option<(i32, f64, f64)> {
    match chunk {
        RawChunk::HeatCapacity { chunk, .. } => Some((
            chunk.phase_id_raw,
            chunk.temperature_min,
            chunk.temperature_max,
        )),
        RawChunk::Kappa(chunk) => Some((
            chunk.phase_id_raw,
            chunk.temperature_min,
            chunk.temperature_max,
        )),
        _ => None,
    }
}

fn validate_range(
    chunk: &RawChunk,
    chunk_index: usize,
    compound_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some((phase_id_raw, t_min, t_max)) = range_bounds(chunk) else {
        return;
    };
    let kind = if !t_min.is_finite() || !t_max.is_finite() {
        Some(DiagnosticKind::NonFiniteTemperatureRange {
            phase_id_raw,
            t_min,
            t_max,
        })
    } else if t_min > t_max {
        Some(DiagnosticKind::TemperatureRangeReversed {
            phase_id_raw,
            t_min,
            t_max,
        })
    } else {
        None
    };
    if let Some(kind) = kind {
        diagnostics.push(Diagnostic {
            chunk_index,
            byte_offset: chunk_index.saturating_mul(CHUNK_SIZE),
            compound_index: Some(compound_index),
            kind,
        });
    }
}

fn attach_range(
    compound: &mut CompoundIndex,
    phase_indices: &HashMap<i32, Vec<usize>>,
    chunk: &RawChunk,
    chunk_index: usize,
    compound_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some((phase_id_raw, _, _)) = range_bounds(chunk) else {
        return;
    };
    let Some(candidates) = phase_indices.get(&phase_id_raw) else {
        let kind = match chunk {
            RawChunk::HeatCapacity { kind, .. } => DiagnosticKind::OrphanHeatCapacityRange {
                phase_id_raw,
                kind: *kind,
            },
            RawChunk::Kappa(_) => DiagnosticKind::OrphanKappaRange { phase_id_raw },
            _ => return,
        };
        diagnostics.push(Diagnostic {
            chunk_index,
            byte_offset: chunk_index.saturating_mul(CHUNK_SIZE),
            compound_index: Some(compound_index),
            kind,
        });
        compound.orphan_ranges.push(OrphanRangeIndex {
            chunk_index,
            byte_offset: chunk_index.saturating_mul(CHUNK_SIZE),
            phase_id_raw,
            reason: OrphanReason::MissingPhase,
        });
        return;
    };

    if candidates.len() != 1 {
        diagnostics.push(Diagnostic {
            chunk_index,
            byte_offset: chunk_index.saturating_mul(CHUNK_SIZE),
            compound_index: Some(compound_index),
            kind: DiagnosticKind::AmbiguousPhaseLink {
                phase_id_raw,
                phase_count: candidates.len(),
            },
        });
        compound.orphan_ranges.push(OrphanRangeIndex {
            chunk_index,
            byte_offset: chunk_index.saturating_mul(CHUNK_SIZE),
            phase_id_raw,
            reason: OrphanReason::AmbiguousPhase {
                phase_count: candidates.len(),
            },
        });
        return;
    }

    match chunk {
        RawChunk::HeatCapacity { .. } => {
            compound.phase_indexes[candidates[0]]
                .heat_capacity_chunks
                .push(chunk_index);
        }
        RawChunk::Kappa(_) => {
            compound.phase_indexes[candidates[0]]
                .kappa_chunks
                .push(chunk_index);
        }
        _ => {}
    }
}
