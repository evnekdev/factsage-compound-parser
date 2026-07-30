use std::fmt;

use crate::thermo::{PhaseKind, UnitError};

/// Errors raised by controlled raw CDB edits.
#[derive(Debug, Clone, PartialEq)]
pub enum EditError {
    /// The requested semantic compound index was not present.
    CompoundNotFound {
        /// Requested zero-based compound index.
        compound_index: usize,
    },
    /// The requested phase index was not present within its compound.
    PhaseNotFound {
        /// Requested zero-based compound index.
        compound_index: usize,
        /// Requested zero-based phase index.
        phase_index: usize,
    },
    /// The requested setter does not apply to the selected phase variant.
    WrongPhaseType {
        /// Compound containing the selected phase.
        compound_index: usize,
        /// Selected phase index.
        phase_index: usize,
        /// Variant required by the setter.
        expected: PhaseKind,
        /// Variant found in the raw stream.
        actual: PhaseKind,
    },
    /// A text value exceeds its fixed-width destination.
    TextTooLong {
        /// Name of the edited fixed-width field.
        field: &'static str,
        /// Maximum number of encoded bytes.
        maximum: usize,
        /// Number of supplied encoded bytes.
        actual: usize,
    },
    /// A text value contains a non-ASCII character.
    NonAsciiText {
        /// Name of the edited fixed-width field.
        field: &'static str,
        /// UTF-8 byte offset of the first non-ASCII character.
        byte_index: usize,
    },
    /// A text value contains an embedded NUL, which would truncate its display value.
    EmbeddedNul {
        /// Name of the edited fixed-width field.
        field: &'static str,
    },
    /// A numeric setter was given a NaN or infinite value.
    NonFiniteValue {
        /// Name of the edited numeric field.
        field: &'static str,
        /// Rejected value.
        value: f64,
    },
    /// A compound unit does not support the requested SI conversion.
    Unit(UnitError),
    /// An internal raw-stream lookup encountered an unexpected chunk variant.
    InconsistentRawStream {
        /// Physical chunk index.
        chunk_index: usize,
    },
}

impl fmt::Display for EditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CompoundNotFound { compound_index } => {
                write!(formatter, "no compound at index {compound_index}")
            }
            Self::PhaseNotFound {
                compound_index,
                phase_index,
            } => write!(
                formatter,
                "no phase at index {phase_index} in compound {compound_index}"
            ),
            Self::WrongPhaseType {
                compound_index,
                phase_index,
                expected,
                actual,
            } => write!(
                formatter,
                "phase {phase_index} in compound {compound_index} is {actual:?}, expected {expected:?}"
            ),
            Self::TextTooLong {
                field,
                maximum,
                actual,
            } => write!(
                formatter,
                "{field} is {actual} bytes; maximum fixed width is {maximum}"
            ),
            Self::NonAsciiText { field, byte_index } => {
                write!(
                    formatter,
                    "{field} contains non-ASCII text at byte {byte_index}"
                )
            }
            Self::EmbeddedNul { field } => write!(formatter, "{field} contains an embedded NUL"),
            Self::NonFiniteValue { field, value } => {
                write!(formatter, "{field} must be finite, got {value}")
            }
            Self::Unit(error) => write!(formatter, "unit conversion error: {error}"),
            Self::InconsistentRawStream { chunk_index } => {
                write!(formatter, "unexpected raw chunk at index {chunk_index}")
            }
        }
    }
}

impl std::error::Error for EditError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unit(error) => Some(error),
            _ => None,
        }
    }
}

impl From<UnitError> for EditError {
    fn from(error: UnitError) -> Self {
        Self::Unit(error)
    }
}
