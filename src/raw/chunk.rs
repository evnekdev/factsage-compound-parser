use super::{Cursor, comment, compound, database_header, heat_capacity, kappa, phase};
use crate::error::ParseError;

/// A typed representation of every known CDB chunk, plus lossless unknown chunks.
#[derive(Debug, Clone, PartialEq)]
pub enum RawChunk {
    /// An ID-1 compound chunk.
    Compound(super::RawCompoundChunk),
    /// An ID-2 through ID-6 heat-capacity chunk with its original kind.
    HeatCapacity {
        /// The original heat-capacity chunk ID.
        kind: super::HeatCapacityKind,
        /// The typed raw CP body.
        chunk: super::RawHeatCapacityChunk,
    },
    /// An ID-7 ordinary phase chunk.
    PhaseOrdinary(super::RawOrdinaryPhaseChunk),
    /// An ID-8 transition phase chunk.
    PhaseTransition(super::RawTransitionPhaseChunk),
    /// The ID-9 database header.
    DatabaseHeader(super::RawDatabaseHeaderChunk),
    /// An ID-10 comment fragment.
    Comment(super::RawCommentChunk),
    /// An ID-11 kappa/extended-property chunk.
    Kappa(super::RawKappaChunk),
    /// An unrecognised ID with its complete 255-byte body preserved.
    Unknown {
        /// The unrecognised chunk ID.
        id: u8,
        /// The uninterpreted chunk body.
        body: [u8; 255],
    },
}

impl RawChunk {
    /// Returns the original one-byte chunk ID.
    pub const fn id(&self) -> u8 {
        match self {
            Self::Compound(_) => 1,
            Self::HeatCapacity { kind, .. } => kind.id(),
            Self::PhaseOrdinary(_) => 7,
            Self::PhaseTransition(_) => 8,
            Self::DatabaseHeader(_) => 9,
            Self::Comment(_) => 10,
            Self::Kappa(_) => 11,
            Self::Unknown { id, .. } => *id,
        }
    }
}

pub(crate) fn parse(chunk_index: usize, chunk_bytes: &[u8]) -> Result<RawChunk, ParseError> {
    if chunk_bytes.len() != 256 {
        return Err(ParseError::TruncatedRecord {
            chunk_index,
            byte_offset: chunk_index * 256,
            expected: 256,
            actual: chunk_bytes.len(),
        });
    }

    let id = chunk_bytes[0];
    let body = &chunk_bytes[1..];
    let mut cursor = Cursor::new(body, chunk_index, chunk_index * 256 + 1, record_type(id));

    match id {
        1 => Ok(RawChunk::Compound(compound::parse(&mut cursor)?)),
        2..=6 => Ok(RawChunk::HeatCapacity {
            kind: heat_capacity::HeatCapacityKind::from_id(id).expect("range 2..=6 is exhaustive"),
            chunk: heat_capacity::parse(&mut cursor)?,
        }),
        7 => Ok(RawChunk::PhaseOrdinary(phase::parse_ordinary(&mut cursor)?)),
        8 => Ok(RawChunk::PhaseTransition(phase::parse_transition(
            &mut cursor,
        )?)),
        9 => Ok(RawChunk::DatabaseHeader(database_header::parse(
            &mut cursor,
            chunk_index,
        )?)),
        10 => Ok(RawChunk::Comment(comment::parse(&mut cursor)?)),
        11 => Ok(RawChunk::Kappa(kappa::parse(&mut cursor)?)),
        _ => {
            let mut raw_body = [0_u8; 255];
            raw_body.copy_from_slice(body);
            Ok(RawChunk::Unknown { id, body: raw_body })
        }
    }
}

const fn record_type(id: u8) -> &'static str {
    match id {
        1 => "compound_body",
        2..=6 => "cp_body",
        7 => "phase_ordinary_body",
        8 => "phase_transition_body",
        9 => "database_header_body",
        10 => "comment_body",
        11 => "kappa_body",
        _ => "unknown_body",
    }
}
