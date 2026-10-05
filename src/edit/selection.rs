//! Native compound-group retention; callers own scientific selection decisions.
use crate::{Database, DomainError, raw::RawEditError};
use std::collections::BTreeSet;

/// Failure to retain the requested native compound groups.
#[derive(Debug)]
pub enum CompoundSelectionError {
    /// A selected encounter position is outside the source database.
    UnknownCompound {
        /// Requested zero-based encounter position.
        index: usize,
        /// Number of source compounds.
        count: usize,
    },
    /// The source or resulting native index is invalid.
    Domain(DomainError),
    /// A structural raw-stream operation failed.
    Raw(RawEditError),
}
impl std::fmt::Display for CompoundSelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCompound { index, count } => {
                write!(f, "compound {index} outside source count {count}")
            }
            Self::Domain(e) => e.fmt(f),
            Self::Raw(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for CompoundSelectionError {}

impl Database {
    /// Retain caller-selected compound groups in source encounter order.
    ///
    /// The caller makes all scientific filtering decisions. This operation owns
    /// only native group boundaries: each compound and its following chunks up
    /// to the next compound. It preserves the preamble and every retained byte,
    /// including unknown chunks, and rebuilds the resulting semantic index.
    /// Repeated positions are harmless; invalid positions are rejected. The
    /// source database is unchanged. Header metadata is preserved verbatim.
    pub fn retain_compound_groups(
        &self,
        positions: &[usize],
    ) -> Result<Self, CompoundSelectionError> {
        let view = self.view().map_err(CompoundSelectionError::Domain)?;
        let starts: Vec<_> = view.compounds().map(|v| v.chunk_index()).collect();
        let selected: BTreeSet<_> = positions.iter().copied().collect();
        if let Some(&index) = selected.iter().find(|&&index| index >= starts.len()) {
            return Err(CompoundSelectionError::UnknownCompound {
                index,
                count: starts.len(),
            });
        }
        let mut raw = self.raw().clone();
        for index in (0..starts.len()).rev() {
            if selected.contains(&index) {
                continue;
            }
            let end = starts.get(index + 1).copied().unwrap_or(raw.chunks().len());
            for chunk in (starts[index]..end).rev() {
                raw.remove_chunk(chunk)
                    .map_err(CompoundSelectionError::Raw)?;
            }
        }
        Self::from_raw(raw).map_err(CompoundSelectionError::Domain)
    }
}
