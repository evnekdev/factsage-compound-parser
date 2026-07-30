use crate::RawChunk;
use crate::raw::{HeatCapacityKind, RawHeatCapacityChunk, RawKappaChunk};

use super::database::OrphanRangeIndex;

/// A borrowed heat-capacity range linked to one phase.
///
/// The original CP ID and every raw field remain available through [`Self::raw`]
/// without copying the 256-byte physical record.
#[derive(Debug, Clone, Copy)]
pub struct HeatCapacityRangeView<'a> {
    kind: HeatCapacityKind,
    raw: &'a RawHeatCapacityChunk,
    chunk_index: usize,
}

impl<'a> HeatCapacityRangeView<'a> {
    pub(crate) const fn new(
        kind: HeatCapacityKind,
        raw: &'a RawHeatCapacityChunk,
        chunk_index: usize,
    ) -> Self {
        Self {
            kind,
            raw,
            chunk_index,
        }
    }

    /// Returns the preserved original CP chunk ID variant.
    pub const fn kind(self) -> HeatCapacityKind {
        self.kind
    }

    /// Returns the zero-based physical chunk index of this range.
    pub const fn chunk_index(self) -> usize {
        self.chunk_index
    }

    /// Returns the complete borrowed raw CP record.
    pub const fn raw(self) -> &'a RawHeatCapacityChunk {
        self.raw
    }
}

/// A borrowed ID-11 record linked to one phase by exact stored raw phase ID.
///
/// The domain link and physical stream order are validated, but the record's
/// physical equation, units, and coefficient meanings remain unverified. Use
/// [`Self::raw`] for the authoritative structural fields; this view performs no
/// kappa evaluation or unit conversion.
#[derive(Debug, Clone, Copy)]
pub struct PhysicalPropertyRangeView<'a> {
    raw: &'a RawKappaChunk,
    chunk_index: usize,
}

impl<'a> PhysicalPropertyRangeView<'a> {
    pub(crate) const fn new(raw: &'a RawKappaChunk, chunk_index: usize) -> Self {
        Self { raw, chunk_index }
    }

    /// Returns the zero-based physical chunk index of this preserved ID-11 record.
    pub const fn chunk_index(self) -> usize {
        self.chunk_index
    }

    /// Returns the complete authoritative raw ID-11 record without copying it.
    pub const fn raw(self) -> &'a RawKappaChunk {
        self.raw
    }
}

/// A borrowed range record retained in either a phase link or orphan collection.
#[derive(Debug, Clone, Copy)]
pub enum RangeView<'a> {
    /// A heat-capacity range with its preserved CP ID.
    HeatCapacity(HeatCapacityRangeView<'a>),
    /// A structurally parsed ID-11 record with unverified physical semantics.
    Kappa(PhysicalPropertyRangeView<'a>),
}

impl RangeView<'_> {
    /// Returns the stored raw phase ID referenced by this range.
    pub const fn phase_id_raw(self) -> i32 {
        match self {
            Self::HeatCapacity(range) => range.raw.phase_id_raw,
            Self::Kappa(range) => range.raw.phase_id_raw,
        }
    }

    /// Returns the zero-based physical chunk index of this range.
    pub const fn chunk_index(self) -> usize {
        match self {
            Self::HeatCapacity(range) => range.chunk_index,
            Self::Kappa(range) => range.chunk_index,
        }
    }
}

/// Why a range could not be attached to exactly one phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrphanReason {
    /// No phase in the current compound had the referenced raw ID.
    MissingPhase,
    /// More than one phase in the current compound had the referenced raw ID.
    AmbiguousPhase {
        /// Number of matching phase records.
        phase_count: usize,
    },
}

/// A borrowed preserved range that was not attached to a phase.
///
/// The range remains in raw stream order and can be inspected through
/// [`Self::range`], which returns `None` only if a caller has somehow supplied a
/// mismatched raw stream instead of the one that built the index.
#[derive(Debug, Clone, Copy)]
pub struct OrphanRangeView<'a> {
    raw_chunks: &'a [RawChunk],
    index: &'a OrphanRangeIndex,
}

impl<'a> OrphanRangeView<'a> {
    pub(crate) const fn new(raw_chunks: &'a [RawChunk], index: &'a OrphanRangeIndex) -> Self {
        Self { raw_chunks, index }
    }

    /// Returns the zero-based physical source chunk index.
    pub const fn chunk_index(self) -> usize {
        self.index.chunk_index
    }

    /// Returns the absolute physical byte offset of the source chunk.
    pub const fn byte_offset(self) -> usize {
        self.index.byte_offset
    }

    /// Returns the stored raw phase ID referenced by the orphan range.
    pub const fn phase_id_raw(self) -> i32 {
        self.index.phase_id_raw
    }

    /// Returns the typed reason that no unique phase link was created.
    pub const fn reason(self) -> &'a OrphanReason {
        &self.index.reason
    }

    /// Returns the complete borrowed range, or `None` for an invalid foreign stream.
    pub fn range(self) -> Option<RangeView<'a>> {
        match self.raw_chunks.get(self.index.chunk_index) {
            Some(RawChunk::HeatCapacity { kind, chunk }) => Some(RangeView::HeatCapacity(
                HeatCapacityRangeView::new(*kind, chunk, self.index.chunk_index),
            )),
            Some(RawChunk::Kappa(chunk)) => Some(RangeView::Kappa(PhysicalPropertyRangeView::new(
                chunk,
                self.index.chunk_index,
            ))),
            _ => None,
        }
    }

    /// Returns the raw range chunk, or `None` for an invalid foreign stream.
    pub fn raw_chunk(self) -> Option<&'a RawChunk> {
        self.raw_chunks.get(self.index.chunk_index)
    }
}
