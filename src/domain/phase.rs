use std::borrow::Cow;

use crate::RawChunk;
use crate::raw::{RawOrdinaryPhaseChunk, RawTransitionPhaseChunk};

use super::database::PhaseIndex;
use super::range::{HeatCapacityRangeView, PhysicalPropertyRangeView};
use super::{TextDecodeError, text::decode_ascii};

/// The phase state inferred from a stored FactSage phase ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseState {
    /// A solid phase.
    Solid,
    /// A liquid phase.
    Liquid,
    /// A gas phase.
    Gas,
    /// An aqueous phase.
    Aqueous,
}

impl PhaseState {
    pub(crate) fn from_raw_id(phase_id_raw: i32) -> Self {
        if phase_id_raw > 990 {
            Self::Aqueous
        } else if phase_id_raw > 900 {
            Self::Gas
        } else if phase_id_raw > 800 {
            Self::Liquid
        } else {
            Self::Solid
        }
    }

    fn compact_prefix(self) -> &'static str {
        match self {
            Self::Solid => "S",
            Self::Liquid => "L",
            Self::Gas => "G",
            Self::Aqueous => "AQ",
        }
    }

    fn chemapp_prefix(self) -> &'static str {
        match self {
            Self::Solid => "s",
            Self::Liquid => "l",
            Self::Gas => "g",
            Self::Aqueous => "aq",
        }
    }
}

/// A borrowed raw phase record selected from a [`PhaseView`].
///
/// It preserves the complete typed raw record without copying it out of the
/// authoritative [`crate::RawDatabase`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RawPhase<'a> {
    /// An ID-7 ordinary phase record.
    Ordinary(&'a RawOrdinaryPhaseChunk),
    /// An ID-8 transition phase record.
    Transition(&'a RawTransitionPhaseChunk),
}

impl RawPhase<'_> {
    /// Returns the original physical chunk ID, either 7 or 8.
    pub const fn id(self) -> u8 {
        match self {
            Self::Ordinary(_) => 7,
            Self::Transition(_) => 8,
        }
    }

    /// Returns the preserved stored phase identifier without recalculating it.
    pub const fn phase_id_raw(self) -> i32 {
        match self {
            Self::Ordinary(chunk) => chunk.phase_id_raw,
            Self::Transition(chunk) => chunk.phase_id_raw,
        }
    }
}

/// A borrowed semantic phase with linked range indexes.
///
/// This value is cheap to copy and does not own a second phase record. It remains
/// valid only for the lifetime of the [`crate::domain::DatabaseView`] that
/// produced it, which prevents use after mutable raw edits.
#[derive(Debug, Clone, Copy)]
pub struct PhaseView<'a> {
    pub(crate) raw: RawPhase<'a>,
    pub(crate) index: &'a PhaseIndex,
    pub(crate) raw_chunks: &'a [RawChunk],
}

impl<'a> PhaseView<'a> {
    pub(crate) const fn new(
        raw: RawPhase<'a>,
        index: &'a PhaseIndex,
        raw_chunks: &'a [RawChunk],
    ) -> Self {
        Self {
            raw,
            index,
            raw_chunks,
        }
    }

    /// Returns the zero-based physical chunk index of this phase record.
    pub const fn chunk_index(self) -> usize {
        self.index.phase_chunk
    }

    /// Returns the complete borrowed raw phase record.
    pub const fn raw(self) -> RawPhase<'a> {
        self.raw
    }

    /// Returns the preserved stored phase identifier without converting it.
    pub const fn phase_id_raw(self) -> i32 {
        self.raw.phase_id_raw()
    }

    /// Decodes the fixed-width phase name as strict ASCII-compatible text.
    ///
    /// Invalid non-ASCII bytes return [`TextDecodeError`] and are never lossy
    /// decoded by this accessor.
    pub fn name(self) -> Result<Cow<'a, str>, TextDecodeError> {
        let bytes = match self.raw {
            RawPhase::Ordinary(chunk) => &chunk.physical.phase_name,
            RawPhase::Transition(chunk) => &chunk.physical.phase_name,
        };
        decode_ascii(bytes, "phase_name")
    }

    /// Infers the phase state using the established raw-ID thresholds.
    pub fn state(self) -> PhaseState {
        PhaseState::from_raw_id(self.phase_id_raw())
    }

    /// Calculates the state-local phase index without clamping unusual values.
    ///
    /// Solid values subtract 100, liquid 800, gas 900, and aqueous 990. A zero
    /// or negative result is retained and reported by domain diagnostics.
    pub fn index(self) -> i32 {
        let raw_id = self.phase_id_raw();
        match self.state() {
            PhaseState::Aqueous => raw_id - 990,
            PhaseState::Gas => raw_id - 900,
            PhaseState::Liquid => raw_id - 800,
            PhaseState::Solid => raw_id - 100,
        }
    }

    /// Returns the compact uppercase label, such as `S1` or `L2`.
    pub fn compact_label(self) -> String {
        format!("{}{}", self.state().compact_prefix(), self.index())
    }

    /// Returns the ChemApp-style lowercase label, such as `s` or `aq2`.
    pub fn chemapp_label(self) -> String {
        let suffix = if self.index() == 1 {
            String::new()
        } else {
            self.index().to_string()
        };
        format!("{}{}", self.state().chemapp_prefix(), suffix)
    }

    /// Returns whether this is an ID-8 transition phase record.
    pub const fn is_transition(self) -> bool {
        matches!(self.raw, RawPhase::Transition(_))
    }

    /// Iterates heat-capacity records linked by exact stored phase ID.
    ///
    /// Iteration follows original stream order and allocates nothing. Ranges that
    /// were orphaned or ambiguous are intentionally absent and remain available
    /// from the owning [`crate::domain::CompoundView`].
    pub fn heat_capacity_ranges(self) -> impl Iterator<Item = HeatCapacityRangeView<'a>> + 'a {
        self.index
            .heat_capacity_chunks
            .iter()
            .filter_map(move |entry| {
                let Some(crate::RawChunk::HeatCapacity { kind, chunk }) =
                    self.raw_chunks.get(*entry)
                else {
                    return None;
                };
                Some(HeatCapacityRangeView::new(*kind, chunk, *entry))
            })
    }

    /// Iterates structurally parsed ID-11 records linked by exact stored phase ID.
    ///
    /// Iteration preserves physical stream order and does not copy raw records.
    /// The link is established, while ID-11 physical equations and units remain
    /// unverified and are intentionally not evaluated by this API.
    pub fn physical_property_ranges(
        self,
    ) -> impl Iterator<Item = PhysicalPropertyRangeView<'a>> + 'a {
        self.index
            .kappa_chunks
            .iter()
            .filter_map(move |chunk_index| {
                let Some(crate::RawChunk::Kappa(chunk)) = self.raw_chunks.get(*chunk_index) else {
                    return None;
                };
                Some(PhysicalPropertyRangeView::new(chunk, *chunk_index))
            })
    }

    /// Returns the number of heat-capacity ranges linked to this phase.
    pub fn heat_capacity_range_count(self) -> usize {
        self.index.heat_capacity_chunks.len()
    }

    /// Returns the number of structurally linked ID-11 records for this phase.
    pub fn physical_property_range_count(self) -> usize {
        self.index.kappa_chunks.len()
    }
}
