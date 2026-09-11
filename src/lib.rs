#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Lossless native parsing, indexed semantic views, and controlled CDB editing.
//!
//! The crate owns one contiguous [RawDatabase] stream and builds lightweight
//! [DomainIndex] values over it. Semantic views borrow both and therefore do
//! not duplicate raw records. Binary parsing and serialization preserve unknown
//! chunks, fixed-width text bytes, padding, reserved fields, and raw float bits.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Read-only semantic grouping and borrowed CDB views.
pub mod domain;
/// Controlled edits over an authoritative raw CDB stream.
pub mod edit;
/// Errors emitted while parsing the fixed-width physical format.
pub mod error;
/// Lossless physical CDB chunk representations and serialization support.
pub mod raw;
/// Read-only thermodynamic decoding and stored-expression evaluation.
pub mod thermo;

/// Re-exports for the semantic grouping and borrowed view layer.
pub use domain::{
    CompoundView, Database, DatabaseError, DatabaseView, Diagnostic, DiagnosticKind, DomainError,
    DomainIndex, ExpectedCategory, HeatCapacityRangeView, OrphanRangeView, OrphanReason,
    PhaseState, PhaseView, PhysicalPropertyRangeView, RangeView, RawPhase, TextDecodeError,
};
/// Re-exports for controlled raw-authoritative editing.
pub use edit::{DatabaseEditor, EditError};
/// Re-export of errors emitted while parsing a CDB byte stream.
pub use error::ParseError;
/// Re-exports for lossless raw chunk access and serialization.
pub use raw::{
    HeatCapacityKind, RawChunk, RawCommentChunk, RawCommonHeader, RawCompoundChunk,
    RawDatabaseHeaderChunk, RawEditError, RawHeatCapacityChunk, RawKappaChunk,
    RawOrdinaryPhaseChunk, RawPhasePhysicalTail, RawTransitionPhaseChunk, SerializeError,
};
/// Re-exports for thermodynamic unit, date, density, and CP evaluation APIs.
pub use thermo::{
    CompoundDatabaseProfileEvidence, DateError, DensityError, EnergyUnit,
    HeatCapacityAnchorSemantics, HeatCapacityError, OleAutomationDate,
    OrdinaryPhaseThermodynamicView, PhaseHeatCapacityRangeView, PhaseKind, PhaseProperty,
    PhaseThermoError, PhaseThermodynamicView, PhaseThermodynamicViewError, PressureUnit,
    STANDARD_REFERENCE_TEMPERATURE_K, TransitionParentRelation, TransitionPhaseThermodynamicView,
    UnitError,
};

/// The fixed size of every CDB record, including its one-byte ID.
pub const CHUNK_SIZE: usize = 256;

/// The fixed size of a CDB body after its one-byte ID.
pub const BODY_SIZE: usize = CHUNK_SIZE - 1;

static NEXT_RAW_DATABASE_ID: AtomicU64 = AtomicU64::new(1);

/// A lossless, contiguous sequence of authoritative owned CDB chunks.
///
/// This is the crate's sole mutable physical representation. It has no lifetime
/// tied to the input source, so it can be edited and written after parsing.
/// Structural mutations are permitted at this low level and invalidate every
/// [`DomainIndex`] built from the stream; callers must rebuild semantic indexes
/// before constructing new [`DatabaseView`] values.
#[derive(Debug)]
pub struct RawDatabase {
    chunks: Vec<RawChunk>,
    identity: u64,
    generation: u64,
}

impl Clone for RawDatabase {
    /// Clones all authoritative raw chunks into an independently indexable stream.
    ///
    /// The clone receives a distinct identity, so a [`DomainIndex`] for the
    /// original stream cannot accidentally be used with it.
    fn clone(&self) -> Self {
        Self::from_chunks(self.chunks.clone())
    }
}

impl PartialEq for RawDatabase {
    /// Compares physical chunks while intentionally ignoring in-memory index tokens.
    fn eq(&self, other: &Self) -> bool {
        self.chunks == other.chunks
    }
}

impl RawDatabase {
    fn from_chunks(chunks: Vec<RawChunk>) -> Self {
        Self {
            chunks,
            identity: NEXT_RAW_DATABASE_ID.fetch_add(1, Ordering::Relaxed),
            generation: 0,
        }
    }

    fn invalidate_indexes(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Returns the raw chunks in their exact physical stream order.
    ///
    /// The returned slice cannot mutate the stream. Every known field, padding
    /// byte, reserved byte, unknown body, and original chunk ID remains available
    /// through these raw values.
    pub fn chunks(&self) -> &[RawChunk] {
        &self.chunks
    }

    /// Returns mutable raw chunks and invalidates all previously built domain indexes.
    ///
    /// This low-level escape hatch permits arbitrary physical changes, including
    /// semantically invalid ordering. Rebuild [`DomainIndex`] before creating a
    /// new semantic view. Existing views cannot coexist with this mutable borrow.
    pub fn chunks_mut(&mut self) -> &mut [RawChunk] {
        self.invalidate_indexes();
        &mut self.chunks
    }

    /// Inserts a raw chunk at a zero-based physical chunk index.
    ///
    /// Insertion preserves the supplied chunk verbatim but does not validate
    /// semantic ordering. It invalidates every domain index on success. Inserting
    /// at `self.chunks().len()` appends the chunk.
    pub fn insert_chunk(&mut self, index: usize, chunk: RawChunk) -> Result<(), RawEditError> {
        let len = self.chunks.len();
        if index > len {
            return Err(RawEditError::IndexOutOfBounds {
                operation: "insert",
                index,
                len,
            });
        }
        self.chunks
            .try_reserve(1)
            .map_err(|_| RawEditError::Allocation {
                additional_chunks: 1,
            })?;
        self.chunks.insert(index, chunk);
        self.invalidate_indexes();
        Ok(())
    }

    /// Appends a raw chunk without validating semantic ordering.
    ///
    /// This convenience method invalidates all domain indexes. Use
    /// [`Self::insert_chunk`] when an out-of-bounds error must be represented.
    pub fn push_chunk(&mut self, chunk: RawChunk) {
        self.chunks.push(chunk);
        self.invalidate_indexes();
    }

    /// Removes and returns the raw chunk at a zero-based physical chunk index.
    ///
    /// Removal preserves the remaining chunks' relative order and invalidates all
    /// domain indexes. It does not guarantee the resulting stream is semantically
    /// valid; rebuild an index to validate it.
    pub fn remove_chunk(&mut self, index: usize) -> Result<RawChunk, RawEditError> {
        let len = self.chunks.len();
        if index >= len {
            return Err(RawEditError::IndexOutOfBounds {
                operation: "remove",
                index,
                len,
            });
        }
        let chunk = self.chunks.remove(index);
        self.invalidate_indexes();
        Ok(chunk)
    }

    /// Parses a complete CDB byte slice.
    ///
    /// The parser validates the fixed record size, the required first
    /// database-header ID, the CMPD magic, and every known body boundary.
    /// The returned stream owns all records and does not retain `bytes`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.is_empty() {
            return Err(ParseError::EmptyFile);
        }
        if bytes.len() % CHUNK_SIZE != 0 {
            return Err(ParseError::InvalidFileLength {
                length: bytes.len(),
                chunk_size: CHUNK_SIZE,
            });
        }
        if bytes[0] != 9 {
            return Err(ParseError::InvalidFirstChunkId {
                found: bytes[0],
                expected: 9,
            });
        }

        let chunk_count = bytes.len() / CHUNK_SIZE;
        let mut chunks = Vec::new();
        chunks
            .try_reserve_exact(chunk_count)
            .map_err(|_| ParseError::Allocation { chunk_count })?;
        for (chunk_index, chunk_bytes) in bytes.chunks_exact(CHUNK_SIZE).enumerate() {
            chunks.push(raw::chunk::parse(chunk_index, chunk_bytes)?);
        }
        Ok(Self::from_chunks(chunks))
    }

    /// Reads and parses CDB records directly from any readable source.
    ///
    /// The reader is consumed as exact 256-byte records and therefore avoids a
    /// second full-file byte buffer. A clean EOF after a complete record ends the
    /// stream; a partial final record returns [`ParseError::TruncatedRecord`].
    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self, ParseError> {
        let mut chunks = Vec::new();
        let mut chunk_index = 0_usize;

        while let Some(record) = read_record(&mut reader, chunk_index)? {
            if chunk_index == 0 && record[0] != 9 {
                return Err(ParseError::InvalidFirstChunkId {
                    found: record[0],
                    expected: 9,
                });
            }
            chunks.try_reserve(1).map_err(|_| ParseError::Allocation {
                chunk_count: chunk_index.saturating_add(1),
            })?;
            chunks.push(raw::chunk::parse(chunk_index, &record)?);
            chunk_index = chunk_index
                .checked_add(1)
                .ok_or(ParseError::RecordCountOverflow)?;
        }

        if chunks.is_empty() {
            return Err(ParseError::EmptyFile);
        }
        Ok(Self::from_chunks(chunks))
    }

    /// Opens and parses a complete CDB from a filesystem path using streaming I/O.
    ///
    /// The path is opened for reading only. An open failure is returned as
    /// [`ParseError::OpenPath`]; an operating-system error after the open is
    /// returned as [`ParseError::ReadPath`]. Format errors retain their normal
    /// typed variants and do not modify the source path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|source| ParseError::OpenPath {
            path: path.to_path_buf(),
            source,
        })?;
        match Self::from_reader(file) {
            Err(ParseError::Io(source)) => Err(ParseError::ReadPath {
                path: path.to_path_buf(),
                source,
            }),
            result => result,
        }
    }

    pub(crate) const fn index_token(&self) -> (u64, u64) {
        (self.identity, self.generation)
    }

    pub(crate) fn chunk_mut_without_invalidation(&mut self, index: usize) -> Option<&mut RawChunk> {
        self.chunks.get_mut(index)
    }
}

fn read_record<R: Read>(
    reader: &mut R,
    chunk_index: usize,
) -> Result<Option<[u8; CHUNK_SIZE]>, ParseError> {
    let mut record = [0_u8; CHUNK_SIZE];
    let mut actual = 0;
    while actual < CHUNK_SIZE {
        match reader.read(&mut record[actual..]) {
            Ok(0) if actual == 0 => return Ok(None),
            Ok(0) => {
                return Err(ParseError::TruncatedRecord {
                    chunk_index,
                    byte_offset: chunk_index.saturating_mul(CHUNK_SIZE),
                    expected: CHUNK_SIZE,
                    actual,
                });
            }
            Ok(read) => actual += read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(ParseError::Io(error)),
        }
    }
    Ok(Some(record))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_format() {
        assert_eq!(CHUNK_SIZE, 256);
        assert_eq!(BODY_SIZE, 255);
    }
}
