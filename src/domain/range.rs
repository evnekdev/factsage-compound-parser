use crate::raw::{HeatCapacityKind, RawHeatCapacityChunk, RawKappaChunk};

/// A heat-capacity range attached to a phase.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatCapacityRange {
    /// The original CP chunk variant.
    pub kind: HeatCapacityKind,
    /// The complete raw CP record.
    pub raw: RawHeatCapacityChunk,
}

/// A kappa or extended physical-property range attached to a phase.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalPropertyRange {
    /// The complete raw kappa record.
    pub raw: RawKappaChunk,
}

/// A range record attached to an orphan or ambiguous reference collection.
#[derive(Debug, Clone, PartialEq)]
pub enum Range {
    /// A heat-capacity range.
    HeatCapacity(HeatCapacityRange),
    /// A kappa range.
    Kappa(PhysicalPropertyRange),
}

impl Range {
    /// Returns the raw phase ID referenced by this range.
    pub fn phase_id_raw(&self) -> i32 {
        match self {
            Self::HeatCapacity(range) => range.raw.phase_id_raw,
            Self::Kappa(range) => range.raw.phase_id_raw,
        }
    }
}

/// Why a range could not be attached to exactly one phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrphanReason {
    /// There was no phase with the referenced raw ID.
    MissingPhase,
    /// More than one phase had the referenced raw ID.
    AmbiguousPhase { phase_count: usize },
}

/// A preserved range that was not attached to a phase.
#[derive(Debug, Clone, PartialEq)]
pub struct OrphanRange {
    /// Zero-based source chunk index.
    pub chunk_index: usize,
    /// Absolute source byte offset.
    pub byte_offset: usize,
    /// Raw phase ID referenced by the source record.
    pub phase_id_raw: i32,
    /// Reason the range was not attached.
    pub reason: OrphanReason,
    /// Complete preserved range record.
    pub range: Range,
}
