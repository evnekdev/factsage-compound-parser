use std::fmt;

/// Errors returned by low-level structural edits to a [`crate::RawDatabase`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawEditError {
    /// A requested physical chunk index was outside the current stream.
    IndexOutOfBounds {
        /// Operation that received the invalid index.
        operation: &'static str,
        /// Requested zero-based physical chunk index.
        index: usize,
        /// Current number of chunks in the stream.
        len: usize,
    },
    /// Reserving storage for an inserted chunk failed.
    Allocation {
        /// Number of chunks the operation attempted to reserve.
        additional_chunks: usize,
    },
}

impl fmt::Display for RawEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexOutOfBounds {
                operation,
                index,
                len,
            } => write!(
                formatter,
                "cannot {operation} at chunk index {index}; stream length is {len}"
            ),
            Self::Allocation { additional_chunks } => write!(
                formatter,
                "could not reserve space for {additional_chunks} additional CDB chunk(s)"
            ),
        }
    }
}

impl std::error::Error for RawEditError {}
