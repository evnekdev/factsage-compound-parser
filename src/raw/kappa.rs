use super::{Cursor, common_header};
use crate::error::ParseError;

/// The ID-11 extended physical-property/kappa body.
#[derive(Debug, Clone, PartialEq)]
pub struct RawKappaChunk {
    /// The shared non-database header.
    pub header: super::RawCommonHeader,
    /// The lower temperature bound.
    pub temperature_min: f64,
    /// The upper temperature bound.
    pub temperature_max: f64,
    /// The referenced phase identifier.
    pub phase_id_raw: i32,
    /// Four uninterpreted bytes.
    pub unknown_1: [u8; 4],
    /// Ten f1 temperature coefficients.
    pub f1_temperature_coefficients: [f64; 10],
    /// Eight f1 temperature powers.
    pub f1_temperature_powers: [f32; 8],
    /// Three f2 pressure coefficients.
    pub f2_pressure_coefficients: [f64; 3],
    /// Two f2 pressure powers.
    pub f2_pressure_powers: [f32; 2],
    /// Five f3 temperature coefficients.
    pub f3_temperature_coefficients: [f64; 5],
    /// Three f3 temperature powers.
    pub f3_temperature_powers: [f32; 3],
    /// Final padding bytes.
    pub padding_remaining: [u8; 4],
}

pub(crate) fn parse(cursor: &mut Cursor<'_>) -> Result<RawKappaChunk, ParseError> {
    let chunk = RawKappaChunk {
        header: common_header::parse(cursor)?,
        temperature_min: cursor.read_f64("temperature_min")?,
        temperature_max: cursor.read_f64("temperature_max")?,
        phase_id_raw: cursor.read_i32("phase_id_raw")?,
        unknown_1: cursor.read_u8_array("unknown_1")?,
        f1_temperature_coefficients: cursor.read_f64_array("f1_temperature_coefficients")?,
        f1_temperature_powers: cursor.read_f32_array("f1_temperature_powers")?,
        f2_pressure_coefficients: cursor.read_f64_array("f2_pressure_coefficients")?,
        f2_pressure_powers: cursor.read_f32_array("f2_pressure_powers")?,
        f3_temperature_coefficients: cursor.read_f64_array("f3_temperature_coefficients")?,
        f3_temperature_powers: cursor.read_f32_array("f3_temperature_powers")?,
        padding_remaining: cursor.read_u8_array("padding_remaining")?,
    };
    cursor.finish()?;
    Ok(chunk)
}
