use std::fmt;

use crate::{error::ParseError, raw::HeatCapacityKind};

use super::phase::PhaseState;

/// An error returned when a fixed-width ASCII field cannot be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDecodeError {
    field: String,
    valid_up_to: usize,
}

impl TextDecodeError {
    pub(crate) fn new(field: &'static str, valid_up_to: usize) -> Self {
        Self {
            field: field.to_owned(),
            valid_up_to,
        }
    }

    /// Returns the logical field name that failed decoding.
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Returns the byte offset at which decoding first failed.
    pub fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }
}

impl fmt::Display for TextDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "field {} contains non-ASCII data at byte {}",
            self.field, self.valid_up_to
        )
    }
}

impl std::error::Error for TextDecodeError {}

/// The category expected by the grouping state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedCategory {
    /// A new compound record.
    Compound,
    /// A phase, range, comment, or the next compound.
    PhaseRangeCommentOrCompound,
    /// A range, comment, or the next compound.
    RangeCommentOrCompound,
    /// A comment or the next compound.
    CommentOrCompound,
}

impl fmt::Display for ExpectedCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Compound => "compound",
            Self::PhaseRangeCommentOrCompound => "phase, range, comment, or compound",
            Self::RangeCommentOrCompound => "range, comment, or compound",
            Self::CommentOrCompound => "comment or compound",
        };
        formatter.write_str(text)
    }
}

/// A structural error encountered while grouping a raw database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// The raw database did not contain any chunks.
    EmptyRawDatabase,
    /// The first raw chunk was not a database header.
    MissingDatabaseHeader { found: Option<u8> },
    /// A known chunk appeared in a state where it is not allowed.
    Ordering {
        chunk_index: usize,
        byte_offset: usize,
        chunk_id: u8,
        expected: ExpectedCategory,
        compound_index: Option<usize>,
    },
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRawDatabase => formatter.write_str("raw database is empty"),
            Self::MissingDatabaseHeader { found } => {
                write!(
                    formatter,
                    "first raw chunk is not a database header: {found:?}"
                )
            }
            Self::Ordering {
                chunk_index,
                byte_offset,
                chunk_id,
                expected,
                compound_index,
            } => write!(
                formatter,
                "invalid chunk ordering at chunk {chunk_index} (byte {byte_offset}), id {chunk_id}; expected {expected}, compound {compound_index:?}"
            ),
        }
    }
}

impl std::error::Error for DomainError {}

/// An error returned by a convenience database entry point.
#[derive(Debug)]
pub enum DatabaseError {
    /// The physical raw parser rejected the input.
    Parse(ParseError),
    /// The raw chunks could not be grouped into the documented stream model.
    Domain(DomainError),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(formatter, "raw parse error: {error}"),
            Self::Domain(error) => write!(formatter, "domain error: {error}"),
        }
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            Self::Domain(error) => Some(error),
        }
    }
}

impl From<ParseError> for DatabaseError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

/// The semantic category of a non-fatal issue.
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticKind {
    /// Multiple phases in one compound use the same raw phase ID.
    DuplicatePhaseId { phase_id_raw: i32 },
    /// A heat-capacity range references no phase.
    OrphanHeatCapacityRange {
        phase_id_raw: i32,
        kind: HeatCapacityKind,
    },
    /// A kappa range references no phase.
    OrphanKappaRange { phase_id_raw: i32 },
    /// A range references more than one phase with the same raw ID.
    AmbiguousPhaseLink {
        phase_id_raw: i32,
        phase_count: usize,
    },
    /// A fixed-width ASCII field contains a non-ASCII byte.
    InvalidAsciiText { field: String },
    /// A phase ID produces a zero or negative calculated index.
    SuspiciousPhaseId {
        phase_id_raw: i32,
        state: PhaseState,
        index: i32,
    },
    /// A range contains a non-finite temperature bound.
    NonFiniteTemperatureRange {
        phase_id_raw: i32,
        t_min: f64,
        t_max: f64,
    },
    /// A range has its upper bound below its lower bound.
    TemperatureRangeReversed {
        phase_id_raw: i32,
        t_min: f64,
        t_max: f64,
    },
    /// An unknown chunk was retained inside a compound group.
    UnknownChunk { id: u8 },
}

/// A non-fatal issue retained alongside the semantic database.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    /// Zero-based chunk index in the original file.
    pub chunk_index: usize,
    /// Absolute byte offset of the chunk in the original file.
    pub byte_offset: usize,
    /// Compound index, when the issue belongs to a compound group.
    pub compound_index: Option<usize>,
    /// Typed issue information.
    pub kind: DiagnosticKind,
}
