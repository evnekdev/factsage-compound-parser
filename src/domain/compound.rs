use std::borrow::Cow;

use crate::raw::{RawChunk, RawCommentChunk, RawCompoundChunk};

use super::{
    error::TextDecodeError,
    phase::Phase,
    range::OrphanRange,
    text::{decode_ascii, decode_windows_1252},
};

/// A semantically grouped compound record.
#[derive(Debug, Clone, PartialEq)]
pub struct Compound {
    /// The complete original compound record.
    pub raw: RawCompoundChunk,
    /// Phases belonging to this compound in stream order.
    pub phases: Vec<Phase>,
    /// Consecutive comment fragments belonging to this compound.
    pub comment_fragments: Vec<RawCommentChunk>,
    /// Preserved ranges that could not be linked uniquely to a phase.
    pub orphan_ranges: Vec<OrphanRange>,
    /// Unknown chunks encountered inside this compound group.
    pub unknown_chunks: Vec<RawChunk>,
}

impl Compound {
    pub(crate) fn new(raw: RawCompoundChunk) -> Self {
        Self {
            raw,
            phases: Vec::new(),
            comment_fragments: Vec::new(),
            orphan_ranges: Vec::new(),
            unknown_chunks: Vec::new(),
        }
    }

    /// Decodes the fixed-width compound name as strict ASCII-compatible text.
    pub fn name(&self) -> Result<Cow<'_, str>, TextDecodeError> {
        decode_ascii(&self.raw.compound_name, "compound_name")
    }

    /// Decodes the fixed-width formula as strict ASCII-compatible text.
    pub fn formula(&self) -> Result<Cow<'_, str>, TextDecodeError> {
        decode_ascii(&self.raw.formula_name, "formula_name")
    }

    /// Returns the original seven real stoichiometric coefficients.
    pub const fn real_stoichiometric_coefficients(&self) -> &[f64; 7] {
        &self.raw.real_stoichiometric_coefficients
    }

    /// Decodes and concatenates comment fragments without adding separators.
    pub fn joined_comment(&self) -> String {
        self.comment_fragments
            .iter()
            .map(|fragment| decode_windows_1252(&fragment.comment))
            .collect()
    }

    /// Finds the first phase with an exact raw phase ID match.
    ///
    /// If duplicate IDs exist, a duplicate diagnostic is present and this helper
    /// returns the first phase in stream order. Use the public phases collection
    /// when all duplicate candidates are needed.
    pub fn find_phase_by_raw_id(&self, phase_id_raw: i32) -> Option<&Phase> {
        self.phases
            .iter()
            .find(|phase| phase.phase_id_raw() == phase_id_raw)
    }

    /// Finds a phase by exact decoded phase name.
    pub fn find_phase_by_name(&self, name: &str) -> Option<&Phase> {
        self.phases
            .iter()
            .find(|phase| phase.name().is_ok_and(|phase_name| phase_name == name))
    }

    /// Finds a phase by exact ChemApp-style label.
    pub fn find_phase_by_chemapp_label(&self, label: &str) -> Option<&Phase> {
        self.phases
            .iter()
            .find(|phase| phase.chemapp_label() == label)
    }

    /// Finds a phase by exact compact uppercase label.
    pub fn find_phase_by_compact_label(&self, label: &str) -> Option<&Phase> {
        self.phases
            .iter()
            .find(|phase| phase.compact_label() == label)
    }
    /// Alias for finding a phase by its ChemApp-style label.
    pub fn find_phase_by_label(&self, label: &str) -> Option<&Phase> {
        self.find_phase_by_chemapp_label(label)
    }
}
