use std::fs::File;
use std::io::Read;
use std::path::Path;

pub mod domain;
pub mod error;
pub mod raw;
pub mod thermo;

pub use domain::{
    Compound, Database, DatabaseError, Diagnostic, DiagnosticKind, DomainError, ExpectedCategory,
    HeatCapacityRange, OrphanRange, OrphanReason, Phase, PhaseDefinition, PhaseState,
    PhysicalPropertyRange, Range, RawPhase, TextDecodeError,
};
pub use error::ParseError;
pub use raw::{
    HeatCapacityKind, RawChunk, RawCommentChunk, RawCommonHeader, RawCompoundChunk,
    RawDatabaseHeaderChunk, RawHeatCapacityChunk, RawKappaChunk, RawOrdinaryPhaseChunk,
    RawPhasePhysicalTail, RawTransitionPhaseChunk,
};
pub use thermo::{
    DateError, DensityError, EnergyUnit, HeatCapacityError, OleAutomationDate, PhaseKind,
    PhaseProperty, PhaseThermoError, PressureUnit, UnitError,
};

/// The fixed size of every CDB record, including its one-byte ID.
pub const CHUNK_SIZE: usize = 256;

/// The fixed size of a CDB body after its one-byte ID.
pub const BODY_SIZE: usize = CHUNK_SIZE - 1;

/// A lossless flat sequence of raw CDB chunks.
#[derive(Debug, Clone, PartialEq)]
pub struct RawDatabase {
    /// The chunks in their original file order.
    pub chunks: Vec<RawChunk>,
}

impl RawDatabase {
    /// Parses a complete CDB byte slice.
    ///
    /// The parser validates the fixed record size, the required first
    /// database-header ID, the CMPD magic, and every known body boundary.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.is_empty() {
            return Err(ParseError::EmptyFile);
        }
        if !bytes.len().is_multiple_of(CHUNK_SIZE) {
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

        let mut chunks = Vec::with_capacity(bytes.len() / CHUNK_SIZE);
        for (chunk_index, chunk_bytes) in bytes.chunks_exact(CHUNK_SIZE).enumerate() {
            chunks.push(raw::chunk::parse(chunk_index, chunk_bytes)?);
        }
        Ok(Self { chunks })
    }

    /// Reads and parses a complete CDB from any readable source.
    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self, ParseError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    /// Opens and parses a complete CDB from a filesystem path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        Self::from_reader(File::open(path)?)
    }
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
