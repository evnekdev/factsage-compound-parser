//! Lossless whole-compound filtering. Phase-level edits require separate
//! reference and counter evidence; this profile never deletes part of a group.

use std::collections::BTreeSet;
use std::fmt;

use crate::{DiagnosticKind, DomainIndex, RawChunk, RawDatabase};

/// Failure to prepare or materialize a dependency-closed compound selection.
#[derive(Debug)]
pub enum CompoundFilterError {
    /// The selected source has invalid known-record ordering.
    Source(String),
    /// A requested zero-based compound index does not exist.
    InvalidCompoundIndex {
        /// Requested source encounter index.
        index: usize,
        /// Number of indexed source groups.
        count: usize,
    },
    /// The selection names one compound more than once.
    DuplicateCompoundIndex {
        /// Repeated source encounter index.
        index: usize,
    },
    /// A retained group has an orphan or ambiguous phase dependency.
    AmbiguousDependency {
        /// Source group with the ambiguous native link.
        compound_index: usize,
        /// Kind of ambiguity requiring explicit repair.
        kind: &'static str,
    },
    /// A sealed plan was applied to another or mutated raw stream.
    StaleSource,
    /// Serialization, reparse or verification rejected the selected stream.
    Verification(String),
}

impl fmt::Display for CompoundFilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CompoundFilterError {}

/// Frozen selection of complete ID-1 groups, keyed by source encounter index.
///
/// Every selected group's phases, CP, ID-8, ID-11, comments and unknown chunks
/// are preserved byte-for-byte. Group-local IDs and counters therefore retain
/// their original valid values; no unsupported CDB allocation rule is inferred.
#[derive(Debug, Clone)]
pub struct CompoundFilterPlan {
    selected: BTreeSet<usize>,
    source_token: (u64, u64),
    source_count: usize,
}

impl CompoundFilterPlan {
    /// Seals an exact source-index selection. An empty selection emits ID-9 only.
    pub fn new(
        source: &RawDatabase,
        selected: impl IntoIterator<Item = usize>,
    ) -> Result<Self, CompoundFilterError> {
        let index = DomainIndex::build(source)
            .map_err(|error| CompoundFilterError::Source(error.to_string()))?;
        let mut selected_indexes = BTreeSet::new();
        for compound_index in selected {
            if !selected_indexes.insert(compound_index) {
                return Err(CompoundFilterError::DuplicateCompoundIndex {
                    index: compound_index,
                });
            }
        }
        let selected = selected_indexes;
        let source_count = index.compound_count();
        if let Some(invalid) = selected
            .iter()
            .find(|candidate| **candidate >= source_count)
        {
            return Err(CompoundFilterError::InvalidCompoundIndex {
                index: *invalid,
                count: source_count,
            });
        }
        for diagnostic in index.diagnostics() {
            let Some(compound_index) = diagnostic.compound_index else {
                continue;
            };
            if !selected.contains(&compound_index) {
                continue;
            }
            let kind = match diagnostic.kind {
                DiagnosticKind::DuplicatePhaseId { .. } => "duplicate phase ID",
                DiagnosticKind::OrphanHeatCapacityRange { .. } => "orphan Cp range",
                DiagnosticKind::OrphanKappaRange { .. } => "orphan ID-11 range",
                DiagnosticKind::AmbiguousPhaseLink { .. } => "ambiguous phase link",
                _ => continue,
            };
            return Err(CompoundFilterError::AmbiguousDependency {
                compound_index,
                kind,
            });
        }
        Ok(Self {
            selected,
            source_token: source.index_token(),
            source_count,
        })
    }

    /// Returns selected source indexes in deterministic source order.
    pub fn selected(&self) -> &BTreeSet<usize> {
        &self.selected
    }

    /// Materializes the complete selected groups, then serializes, reparses and
    /// verifies exact physical equality with the expected retained records.
    pub fn materialize(&self, source: &RawDatabase) -> Result<RawDatabase, CompoundFilterError> {
        if source.index_token() != self.source_token {
            return Err(CompoundFilterError::StaleSource);
        }
        let mut expected = Vec::new();
        let Some(header @ RawChunk::DatabaseHeader(_)) = source.chunks().first() else {
            return Err(CompoundFilterError::Source("missing ID-9".into()));
        };
        expected.push(header.clone());
        let mut current_index = 0;
        let mut retain = false;
        for chunk in source.chunks().iter().skip(1) {
            if matches!(chunk, RawChunk::Compound(_)) {
                retain = self.selected.contains(&current_index);
                current_index += 1;
            }
            if retain {
                expected.push(chunk.clone());
            }
        }
        if current_index != self.source_count {
            return Err(CompoundFilterError::StaleSource);
        }
        let output = RawDatabase::from_chunks(expected.clone());
        let bytes = output
            .to_bytes()
            .map_err(|error| CompoundFilterError::Verification(error.to_string()))?;
        let reparsed = RawDatabase::from_bytes(&bytes)
            .map_err(|error| CompoundFilterError::Verification(error.to_string()))?;
        let index = DomainIndex::build(&reparsed)
            .map_err(|error| CompoundFilterError::Verification(error.to_string()))?;
        if index.compound_count() != self.selected.len() {
            return Err(CompoundFilterError::Verification(
                "selected group count changed on reparse".into(),
            ));
        }
        let verified_bytes = reparsed
            .to_bytes()
            .map_err(|error| CompoundFilterError::Verification(error.to_string()))?;
        if verified_bytes != bytes {
            return Err(CompoundFilterError::Verification(
                "selected records changed on reparse".into(),
            ));
        }
        Ok(output)
    }
}
