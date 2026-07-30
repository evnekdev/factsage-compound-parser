use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::raw::{RawChunk, RawDatabaseHeaderChunk};
use crate::{CHUNK_SIZE, RawDatabase};

use super::compound::Compound;
use super::error::{DatabaseError, Diagnostic, DiagnosticKind, DomainError, ExpectedCategory};
use super::phase::{Phase, RawPhase};
use super::range::{HeatCapacityRange, OrphanRange, OrphanReason, PhysicalPropertyRange, Range};
use super::text::decode_ascii;

/// A read-only semantic database grouped from a flat raw chunk stream.
#[derive(Debug, Clone, PartialEq)]
pub struct Database {
    /// The required first database-header record.
    pub header: RawDatabaseHeaderChunk,
    /// Compounds in original stream order.
    pub compounds: Vec<Compound>,
    /// Non-fatal structural and semantic issues.
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupStage {
    Phases,
    Ranges,
    Comments,
}

impl Database {
    /// Builds the semantic model from an already parsed raw database.
    pub fn from_raw(raw: RawDatabase) -> Result<Self, DomainError> {
        let mut chunks = raw.chunks.into_iter().enumerate().peekable();
        let Some((header_index, first_chunk)) = chunks.next() else {
            return Err(DomainError::EmptyRawDatabase);
        };

        let header = match first_chunk {
            RawChunk::DatabaseHeader(header) => header,
            other => {
                return Err(DomainError::MissingDatabaseHeader {
                    found: Some(other.id()),
                });
            }
        };

        let mut diagnostics = Vec::new();
        validate_ascii_field(
            &header.comment,
            "database_header.comment",
            header_index,
            None,
            &mut diagnostics,
        );

        let mut compounds = Vec::new();
        while let Some((chunk_index, chunk)) = chunks.next() {
            if chunk.id() != 1 {
                return Err(ordering_error(
                    chunk_index,
                    chunk.id(),
                    ExpectedCategory::Compound,
                    None,
                ));
            }

            let RawChunk::Compound(raw_compound) = chunk else {
                unreachable!("chunk ID 1 is always represented by RawChunk::Compound");
            };
            let compound_index = compounds.len();
            let mut compound = Compound::new(raw_compound);
            validate_compound_text(&compound, chunk_index, compound_index, &mut diagnostics);

            let mut phase_indices: HashMap<i32, Vec<usize>> = HashMap::new();
            let mut stage = GroupStage::Phases;

            while let Some((next_index, next_chunk)) = chunks.peek() {
                let next_id = next_chunk.id();

                if next_id == 1 {
                    break;
                }

                if !is_known_id(next_id) {
                    let Some((unknown_index, unknown_chunk)) = chunks.next() else {
                        break;
                    };
                    diagnostics.push(Diagnostic {
                        chunk_index: unknown_index,
                        byte_offset: unknown_index * CHUNK_SIZE,
                        compound_index: Some(compound_index),
                        kind: DiagnosticKind::UnknownChunk {
                            id: unknown_chunk.id(),
                        },
                    });
                    compound.unknown_chunks.push(unknown_chunk);
                    continue;
                }

                match stage {
                    GroupStage::Phases if is_phase_id(next_id) => {
                        let Some((phase_chunk_index, phase_chunk)) = chunks.next() else {
                            break;
                        };
                        let raw_phase = match phase_chunk {
                            RawChunk::PhaseOrdinary(chunk) => RawPhase::Ordinary(chunk),
                            RawChunk::PhaseTransition(chunk) => RawPhase::Transition(chunk),
                            _ => unreachable!("phase ID has a phase representation"),
                        };
                        let phase = Phase::from_raw(raw_phase);
                        validate_phase(&phase, phase_chunk_index, compound_index, &mut diagnostics);
                        let phase_index = compound.phases.len();
                        let phase_id_raw = phase.phase_id_raw();
                        let candidates = phase_indices.entry(phase_id_raw).or_default();
                        candidates.push(phase_index);
                        if candidates.len() > 1 {
                            diagnostics.push(Diagnostic {
                                chunk_index: phase_chunk_index,
                                byte_offset: phase_chunk_index * CHUNK_SIZE,
                                compound_index: Some(compound_index),
                                kind: DiagnosticKind::DuplicatePhaseId { phase_id_raw },
                            });
                        }
                        compound.phases.push(phase);
                    }
                    GroupStage::Phases if is_range_id(next_id) => {
                        stage = GroupStage::Ranges;
                    }
                    GroupStage::Phases if next_id == 10 => {
                        stage = GroupStage::Comments;
                    }
                    GroupStage::Phases => {
                        return Err(ordering_error(
                            *next_index,
                            next_id,
                            ExpectedCategory::PhaseRangeCommentOrCompound,
                            Some(compound_index),
                        ));
                    }
                    GroupStage::Ranges if is_range_id(next_id) => {
                        let Some((range_chunk_index, range_chunk)) = chunks.next() else {
                            break;
                        };
                        let range = raw_range(range_chunk);
                        validate_range(&range, range_chunk_index, compound_index, &mut diagnostics);
                        attach_range(
                            &mut compound,
                            &phase_indices,
                            range,
                            range_chunk_index,
                            compound_index,
                            &mut diagnostics,
                        );
                    }
                    GroupStage::Ranges if next_id == 10 => {
                        stage = GroupStage::Comments;
                    }
                    GroupStage::Ranges => {
                        return Err(ordering_error(
                            *next_index,
                            next_id,
                            ExpectedCategory::RangeCommentOrCompound,
                            Some(compound_index),
                        ));
                    }
                    GroupStage::Comments if next_id == 10 => {
                        let Some((_, comment_chunk)) = chunks.next() else {
                            break;
                        };
                        let RawChunk::Comment(comment) = comment_chunk else {
                            unreachable!("comment ID has a comment representation");
                        };
                        compound.comment_fragments.push(comment);
                    }
                    GroupStage::Comments => {
                        return Err(ordering_error(
                            *next_index,
                            next_id,
                            ExpectedCategory::CommentOrCompound,
                            Some(compound_index),
                        ));
                    }
                }
            }

            compounds.push(compound);
        }

        Ok(Self {
            header,
            compounds,
            diagnostics,
        })
    }

    /// Parses bytes with the raw parser and then builds the domain model.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DatabaseError> {
        Self::from_raw(RawDatabase::from_bytes(bytes)?).map_err(DatabaseError::Domain)
    }

    /// Reads bytes from a reader and then builds the domain model.
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, DatabaseError> {
        Self::from_raw(RawDatabase::from_reader(reader)?).map_err(DatabaseError::Domain)
    }

    /// Opens a path and then builds the domain model.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, DatabaseError> {
        Self::from_reader(File::open(path).map_err(crate::error::ParseError::from)?)
    }

    /// Finds the first compound with an exact decoded formula.
    pub fn find_compound_by_formula(&self, formula: &str) -> Option<&Compound> {
        self.compounds
            .iter()
            .find(|compound| compound.formula().is_ok_and(|value| value == formula))
    }

    /// Finds the first compound whose decoded name starts with the prefix.
    pub fn find_compound_by_name(&self, prefix: &str) -> Option<&Compound> {
        self.compounds
            .iter()
            .find(|compound| compound.name().is_ok_and(|value| value.starts_with(prefix)))
    }
}

impl TryFrom<RawDatabase> for Database {
    type Error = DomainError;

    fn try_from(raw: RawDatabase) -> Result<Self, Self::Error> {
        Self::from_raw(raw)
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
        byte_offset: chunk_index * CHUNK_SIZE,
        chunk_id,
        expected,
        compound_index,
    }
}

fn validate_compound_text(
    compound: &Compound,
    chunk_index: usize,
    compound_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Err(error) = compound.name() {
        diagnostics.push(invalid_text_diagnostic(
            chunk_index,
            compound_index,
            error.field().to_owned(),
        ));
    }
    if let Err(error) = compound.formula() {
        diagnostics.push(invalid_text_diagnostic(
            chunk_index,
            compound_index,
            error.field().to_owned(),
        ));
    }
}

fn validate_phase(
    phase: &Phase,
    chunk_index: usize,
    compound_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Err(error) = phase.name() {
        diagnostics.push(invalid_text_diagnostic(
            chunk_index,
            compound_index,
            error.field().to_owned(),
        ));
    }
    let state = phase.state();
    let index = phase.index();
    if index <= 0 {
        diagnostics.push(Diagnostic {
            chunk_index,
            byte_offset: chunk_index * CHUNK_SIZE,
            compound_index: Some(compound_index),
            kind: DiagnosticKind::SuspiciousPhaseId {
                phase_id_raw: phase.phase_id_raw(),
                state,
                index,
            },
        });
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
            byte_offset: chunk_index * CHUNK_SIZE,
            compound_index,
            kind: DiagnosticKind::InvalidAsciiText {
                field: field.to_owned(),
            },
        });
    }
}

fn invalid_text_diagnostic(chunk_index: usize, compound_index: usize, field: String) -> Diagnostic {
    Diagnostic {
        chunk_index,
        byte_offset: chunk_index * CHUNK_SIZE,
        compound_index: Some(compound_index),
        kind: DiagnosticKind::InvalidAsciiText { field },
    }
}

fn raw_range(chunk: RawChunk) -> Range {
    match chunk {
        RawChunk::HeatCapacity { kind, chunk } => {
            Range::HeatCapacity(HeatCapacityRange { kind, raw: chunk })
        }
        RawChunk::Kappa(chunk) => Range::Kappa(PhysicalPropertyRange { raw: chunk }),
        _ => unreachable!("range ID has a range representation"),
    }
}

fn range_bounds(range: &Range) -> (i32, f64, f64) {
    match range {
        Range::HeatCapacity(range) => (
            range.raw.phase_id_raw,
            range.raw.temperature_min,
            range.raw.temperature_max,
        ),
        Range::Kappa(range) => (
            range.raw.phase_id_raw,
            range.raw.temperature_min,
            range.raw.temperature_max,
        ),
    }
}

fn validate_range(
    range: &Range,
    chunk_index: usize,
    compound_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (phase_id_raw, t_min, t_max) = range_bounds(range);
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
            byte_offset: chunk_index * CHUNK_SIZE,
            compound_index: Some(compound_index),
            kind,
        });
    }
}

fn attach_range(
    compound: &mut Compound,
    phase_indices: &HashMap<i32, Vec<usize>>,
    range: Range,
    chunk_index: usize,
    compound_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let phase_id_raw = range.phase_id_raw();
    let Some(candidates) = phase_indices.get(&phase_id_raw) else {
        let kind = match &range {
            Range::HeatCapacity(range) => DiagnosticKind::OrphanHeatCapacityRange {
                phase_id_raw,
                kind: range.kind,
            },
            Range::Kappa(_) => DiagnosticKind::OrphanKappaRange { phase_id_raw },
        };
        diagnostics.push(Diagnostic {
            chunk_index,
            byte_offset: chunk_index * CHUNK_SIZE,
            compound_index: Some(compound_index),
            kind,
        });
        compound.orphan_ranges.push(OrphanRange {
            chunk_index,
            byte_offset: chunk_index * CHUNK_SIZE,
            phase_id_raw,
            reason: OrphanReason::MissingPhase,
            range,
        });
        return;
    };

    if candidates.len() != 1 {
        diagnostics.push(Diagnostic {
            chunk_index,
            byte_offset: chunk_index * CHUNK_SIZE,
            compound_index: Some(compound_index),
            kind: DiagnosticKind::AmbiguousPhaseLink {
                phase_id_raw,
                phase_count: candidates.len(),
            },
        });
        compound.orphan_ranges.push(OrphanRange {
            chunk_index,
            byte_offset: chunk_index * CHUNK_SIZE,
            phase_id_raw,
            reason: OrphanReason::AmbiguousPhase {
                phase_count: candidates.len(),
            },
            range,
        });
        return;
    }

    match range {
        Range::HeatCapacity(range) => {
            compound.phases[candidates[0]]
                .heat_capacity_ranges
                .push(range);
        }
        Range::Kappa(range) => {
            compound.phases[candidates[0]]
                .physical_property_ranges
                .push(range);
        }
    }
}
