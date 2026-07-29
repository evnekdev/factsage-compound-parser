use super::{Cursor, common_header};
use crate::error::ParseError;

/// The preserved original ID of a heat-capacity range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatCapacityKind {
    /// Original chunk ID 2.
    Id2,
    /// Original chunk ID 3.
    Id3,
    /// Original chunk ID 4.
    Id4,
    /// Original chunk ID 5.
    Id5,
    /// Original chunk ID 6.
    Id6,
}

impl HeatCapacityKind {
    /// Returns the original CDB chunk ID.
    pub const fn id(self) -> u8 {
        match self {
            Self::Id2 => 2,
            Self::Id3 => 3,
            Self::Id4 => 4,
            Self::Id5 => 5,
            Self::Id6 => 6,
        }
    }

    pub(crate) const fn from_id(id: u8) -> Option<Self> {
        match id {
            2 => Some(Self::Id2),
            3 => Some(Self::Id3),
            4 => Some(Self::Id4),
            5 => Some(Self::Id5),
            6 => Some(Self::Id6),
            _ => None,
        }
    }
}

/// The common raw body of CP IDs 2 through 6.
#[derive(Debug, Clone, PartialEq)]
pub struct RawHeatCapacityChunk {
    /// The shared non-database header.
    pub header: super::RawCommonHeader,
    /// The stored enthalpy anchor.
    pub enthalpy: f64,
    /// The stored entropy anchor.
    pub entropy: f64,
    /// The referenced phase identifier.
    pub phase_id_raw: i32,
    /// Four uninterpreted bytes.
    pub unknown_1: [u8; 4],
    /// The lower temperature bound.
    pub temperature_min: f64,
    /// The upper temperature bound.
    pub temperature_max: f64,
    /// Eight heat-capacity coefficients.
    pub coefficients: [f64; 8],
    /// Eight heat-capacity powers.
    pub powers: [f64; 8],
    /// Remaining padding bytes.
    pub padding_remaining: [u8; 56],
}

pub(crate) fn parse(cursor: &mut Cursor<'_>) -> Result<RawHeatCapacityChunk, ParseError> {
    let chunk = RawHeatCapacityChunk {
        header: common_header::parse(cursor)?,
        enthalpy: cursor.read_f64("enthalpy")?,
        entropy: cursor.read_f64("entropy")?,
        phase_id_raw: cursor.read_i32("phase_id_raw")?,
        unknown_1: cursor.read_u8_array("unknown_1")?,
        temperature_min: cursor.read_f64("temperature_min")?,
        temperature_max: cursor.read_f64("temperature_max")?,
        coefficients: cursor.read_f64_array("coefficients")?,
        powers: cursor.read_f64_array("powers")?,
        padding_remaining: cursor.read_u8_array("padding_remaining")?,
    };
    cursor.finish()?;
    Ok(chunk)
}
