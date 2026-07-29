use super::{Cursor, common_header};
use crate::error::ParseError;

/// The ID-1 compound body in its lossless raw representation.
#[derive(Debug, Clone, PartialEq)]
pub struct RawCompoundChunk {
    /// The shared non-database header.
    pub header: super::RawCommonHeader,
    /// The fixed-width compound name bytes.
    pub compound_name: [u8; 40],
    /// The first reserved fixed-width string.
    pub reserved_string_1: [u8; 40],
    /// The fixed-width formula bytes.
    pub formula_name: [u8; 40],
    /// Four uninterpreted bytes.
    pub unknown: [u8; 4],
    /// The raw energy-unit code.
    pub unit_energy: u32,
    /// The raw pressure-unit code.
    pub unit_pressure: u32,
    /// The second reserved fixed-width string.
    pub reserved_string_2: [u8; 12],
    /// Seven real stoichiometric coefficients.
    pub real_stoichiometric_coefficients: [f64; 7],
    /// Final compound padding.
    pub padding_final: [u8; 24],
}

pub(crate) fn parse(cursor: &mut Cursor<'_>) -> Result<RawCompoundChunk, ParseError> {
    let chunk = RawCompoundChunk {
        header: common_header::parse(cursor)?,
        compound_name: cursor.read_u8_array("compound_name")?,
        reserved_string_1: cursor.read_u8_array("reserved_string_1")?,
        formula_name: cursor.read_u8_array("formula_name")?,
        unknown: cursor.read_u8_array("unknown")?,
        unit_energy: cursor.read_u32("unit_energy")?,
        unit_pressure: cursor.read_u32("unit_pressure")?,
        reserved_string_2: cursor.read_u8_array("reserved_string_2")?,
        real_stoichiometric_coefficients: cursor
            .read_f64_array("real_stoichiometric_coefficients")?,
        padding_final: cursor.read_u8_array("padding_final")?,
    };
    cursor.finish()?;
    Ok(chunk)
}

impl RawCompoundChunk {
    /// Returns the compound name with fixed-width padding removed.
    pub fn compound_name_lossy(&self) -> String {
        super::decode_ascii_lossy(&self.compound_name)
    }

    /// Returns the formula bytes as a lossy display string.
    pub fn formula_name_lossy(&self) -> String {
        super::decode_ascii_lossy(&self.formula_name)
    }

    /// Returns the first reserved string as a lossy display string.
    pub fn reserved_string_1_lossy(&self) -> String {
        super::decode_ascii_lossy(&self.reserved_string_1)
    }

    /// Returns the second reserved string as a lossy display string.
    pub fn reserved_string_2_lossy(&self) -> String {
        super::decode_ascii_lossy(&self.reserved_string_2)
    }
}
