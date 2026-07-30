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

    /// Returns the logical fixed-width field that failed ASCII validation.
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Returns the byte offset inside the fixed-width field at which decoding failed.
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

/// The category expected by the semantic grouping state machine.
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

/// A structural or index-coherency error encountered while building semantic views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// The raw database did not contain any chunks.
    EmptyRawDatabase,
    /// The first raw chunk was not a database header.
    MissingDatabaseHeader {
        /// One-byte ID found in the first chunk, or `None` for an empty stream.
        found: Option<u8>,
    },
    /// A known chunk appeared in a state where it is not allowed.
    Ordering {
        /// Zero-based physical chunk index.
        chunk_index: usize,
        /// Absolute physical byte offset.
        byte_offset: usize,
        /// Observed CDB chunk ID.
        chunk_id: u8,
        /// Category permitted by the grouping state machine.
        expected: ExpectedCategory,
        /// Owning compound index when the error occurs inside a compound group.
        compound_index: Option<usize>,
    },
    /// An index belongs to another raw stream or predates a structural mutation.
    StaleIndex {
        /// Identity token captured while building the index.
        index_identity: u64,
        /// Generation token captured while building the index.
        index_generation: u64,
        /// Identity token of the supplied raw stream.
        raw_identity: u64,
        /// Current generation token of the supplied raw stream.
        raw_generation: u64,
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
            Self::StaleIndex {
                index_identity,
                index_generation,
                raw_identity,
                raw_generation,
            } => write!(
                formatter,
                "domain index ({index_identity}, {index_generation}) does not match raw stream ({raw_identity}, {raw_generation})"
            ),
        }
    }
}

impl std::error::Error for DomainError {}

/// An error returned by convenience APIs that parse and then index a database.
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

/// The semantic category of a non-fatal issue retained by a [`super::DomainIndex`].
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticKind {
    /// Multiple phases in one compound use the same raw phase ID.
    DuplicatePhaseId {
        /// The duplicated stored phase identifier.
        phase_id_raw: i32,
    },
    /// A heat-capacity range references no phase.
    OrphanHeatCapacityRange {
        /// Stored phase identifier requested by the range.
        phase_id_raw: i32,
        /// Preserved original CP chunk ID.
        kind: HeatCapacityKind,
    },
    /// A kappa range references no phase.
    OrphanKappaRange {
        /// Stored phase identifier requested by the range.
        phase_id_raw: i32,
    },
    /// A range references more than one phase with the same raw ID.
    AmbiguousPhaseLink {
        /// Stored phase identifier requested by the range.
        phase_id_raw: i32,
        /// Number of candidate phases in the same compound.
        phase_count: usize,
    },
    /// A fixed-width ASCII field contains a non-ASCII byte.
    InvalidAsciiText {
        /// Logical field name whose bytes were invalid.
        field: String,
    },
    /// A phase ID produces a zero or negative calculated state-local index.
    SuspiciousPhaseId {
        /// Preserved stored phase identifier.
        phase_id_raw: i32,
        /// State inferred from the documented ID thresholds.
        state: PhaseState,
        /// State-local index calculated without clamping.
        index: i32,
    },
    /// A range contains a non-finite temperature bound.
    NonFiniteTemperatureRange {
        /// Stored phase identifier requested by the range.
        phase_id_raw: i32,
        /// Preserved lower bound.
        t_min: f64,
        /// Preserved upper bound.
        t_max: f64,
    },
    /// A range has its upper bound below its lower bound.
    TemperatureRangeReversed {
        /// Stored phase identifier requested by the range.
        phase_id_raw: i32,
        /// Preserved lower bound.
        t_min: f64,
        /// Preserved upper bound.
        t_max: f64,
    },
    /// An unknown chunk was retained inside a compound group.
    UnknownChunk {
        /// Unrecognised one-byte chunk ID.
        id: u8,
    },
}

/// A non-fatal issue retained alongside its physical source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    /// Zero-based physical chunk index in the authoritative raw stream.
    pub chunk_index: usize,
    /// Absolute byte offset in the authoritative raw stream.
    pub byte_offset: usize,
    /// Compound index when the issue belongs to a grouped compound.
    pub compound_index: Option<usize>,
    /// Typed description of the detected issue.
    pub kind: DiagnosticKind,
}
