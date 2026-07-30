use std::borrow::Cow;

use crate::{RawChunk, RawCommentChunk, RawCompoundChunk};

use super::database::CompoundIndex;
use super::phase::PhaseView;
use super::range::OrphanRangeView;
use super::{
    TextDecodeError,
    text::{decode_ascii, decode_windows_1252},
};

/// A borrowed semantic compound view over one authoritative raw compound chunk.
///
/// It carries only references and chunk indexes. Names, formulae, comments,
/// phases, unknown chunks, and orphan ranges are all read directly from the raw
/// stream without cloning raw records.
#[derive(Debug, Clone, Copy)]
pub struct CompoundView<'a> {
    pub(crate) raw: &'a RawCompoundChunk,
    pub(crate) index: &'a CompoundIndex,
    pub(crate) raw_chunks: &'a [RawChunk],
}

impl<'a> CompoundView<'a> {
    pub(crate) const fn new(
        raw: &'a RawCompoundChunk,
        index: &'a CompoundIndex,
        raw_chunks: &'a [RawChunk],
    ) -> Self {
        Self {
            raw,
            index,
            raw_chunks,
        }
    }

    /// Returns the zero-based physical chunk index of this compound record.
    pub const fn chunk_index(self) -> usize {
        self.index.compound_chunk
    }

    /// Returns the complete raw compound record preserved in the authoritative stream.
    pub const fn raw(self) -> &'a RawCompoundChunk {
        self.raw
    }

    /// Decodes the fixed-width compound name as strict ASCII-compatible text.
    ///
    /// Invalid bytes produce [`TextDecodeError`] instead of lossy replacement.
    pub fn name(self) -> Result<Cow<'a, str>, TextDecodeError> {
        decode_ascii(&self.raw.compound_name, "compound_name")
    }

    /// Decodes the fixed-width formula as strict ASCII-compatible text.
    pub fn formula(self) -> Result<Cow<'a, str>, TextDecodeError> {
        decode_ascii(&self.raw.formula_name, "formula_name")
    }

    /// Returns the original seven real stoichiometric coefficients.
    ///
    /// Values are raw stored `f64` fields and have no additional interpretation
    /// or unit conversion in this semantic view.
    pub const fn real_stoichiometric_coefficients(self) -> &'a [f64; 7] {
        &self.raw.real_stoichiometric_coefficients
    }

    /// Iterates consecutive Windows-1252 comment fragments in stream order.
    ///
    /// Each fragment is decoded safely after conservative storage-padding trim;
    /// Windows-1252 decoding is infallible and may allocate a decoded string.
    pub fn comment_fragments(self) -> impl Iterator<Item = &'a RawCommentChunk> + 'a {
        self.index
            .comment_chunks
            .iter()
            .filter_map(move |chunk_index| {
                let Some(RawChunk::Comment(comment)) = self.raw_chunks.get(*chunk_index) else {
                    return None;
                };
                Some(comment)
            })
    }

    /// Decodes and concatenates comment fragments without inserting separators.
    ///
    /// Storage padding is removed per fragment; bytes stored as spaces or line
    /// breaks remain exactly as represented by the file.
    pub fn joined_comment(self) -> String {
        self.comment_fragments()
            .map(|fragment| decode_windows_1252(&fragment.comment))
            .collect()
    }

    /// Iterates phases belonging to this compound in original stream order.
    pub fn phases(self) -> impl Iterator<Item = PhaseView<'a>> + 'a {
        self.index.phase_indexes.iter().filter_map(move |index| {
            let raw = match self.raw_chunks.get(index.phase_chunk) {
                Some(RawChunk::PhaseOrdinary(chunk)) => super::RawPhase::Ordinary(chunk),
                Some(RawChunk::PhaseTransition(chunk)) => super::RawPhase::Transition(chunk),
                _ => return None,
            };
            Some(PhaseView::new(raw, index, self.raw_chunks))
        })
    }

    /// Returns the number of phases indexed for this compound.
    pub fn phase_count(self) -> usize {
        self.index.phase_indexes.len()
    }

    /// Iterates raw chunks with unrecognised IDs retained inside this group.
    pub fn unknown_chunks(self) -> impl Iterator<Item = &'a RawChunk> + 'a {
        self.index
            .unknown_chunks
            .iter()
            .filter_map(move |chunk_index| self.raw_chunks.get(*chunk_index))
    }

    /// Iterates preserved ranges that could not be linked to exactly one phase.
    pub fn orphan_ranges(self) -> impl Iterator<Item = OrphanRangeView<'a>> + 'a {
        self.index
            .orphan_ranges
            .iter()
            .map(move |index| OrphanRangeView::new(self.raw_chunks, index))
    }

    /// Returns the number of ranges preserved outside phase links.
    pub fn orphan_range_count(self) -> usize {
        self.index.orphan_ranges.len()
    }

    /// Finds the first phase with an exact stored raw phase ID.
    ///
    /// Duplicate IDs remain diagnosed and preserved; this helper deliberately
    /// returns the first phase in stream order for lookup compatibility.
    pub fn find_phase_by_raw_id(self, phase_id_raw: i32) -> Option<PhaseView<'a>> {
        self.phases()
            .find(|phase| phase.phase_id_raw() == phase_id_raw)
    }

    /// Finds the first phase with an exact decoded phase name.
    pub fn find_phase_by_name(self, name: &str) -> Option<PhaseView<'a>> {
        self.phases()
            .find(|phase| phase.name().is_ok_and(|phase_name| phase_name == name))
    }

    /// Finds the first phase with an exact ChemApp-style label.
    pub fn find_phase_by_chemapp_label(self, label: &str) -> Option<PhaseView<'a>> {
        self.phases().find(|phase| phase.chemapp_label() == label)
    }

    /// Finds the first phase with an exact compact uppercase label.
    pub fn find_phase_by_compact_label(self, label: &str) -> Option<PhaseView<'a>> {
        self.phases().find(|phase| phase.compact_label() == label)
    }

    /// Alias for [`Self::find_phase_by_chemapp_label`].
    pub fn find_phase_by_label(self, label: &str) -> Option<PhaseView<'a>> {
        self.find_phase_by_chemapp_label(label)
    }
}
