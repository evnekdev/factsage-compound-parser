use super::{Cursor, common_header};
use crate::error::ParseError;

/// The losslessly parsed ID-11 extended physical-property record.
///
/// ID 11 is structurally validated and retained exactly. Its phase-ID link is
/// established by the domain layer, but its physical equation, units, and the
/// semantic role of the coefficient arrays remain unverified. The raw fields
/// below are authoritative and must not be treated as evaluated kappa values.
#[derive(Debug, Clone, PartialEq)]
pub struct RawKappaChunk {
    /// The shared non-database header.
    pub header: super::RawCommonHeader,
    /// The stored lower temperature bound; its physical interpretation is unverified.
    pub temperature_min: f64,
    /// The stored upper temperature bound; its physical interpretation is unverified.
    pub temperature_max: f64,
    /// The stored raw phase identifier used for exact in-compound linking.
    pub phase_id_raw: i32,
    /// Four uninterpreted bytes.
    pub unknown_1: [u8; 4],
    /// Ten preserved f64 slots from the first ID-11 coefficient block.
    pub f1_temperature_coefficients: [f64; 10],
    /// Eight preserved f32 slots from the first ID-11 power block.
    pub f1_temperature_powers: [f32; 8],
    /// Three preserved f64 slots from the second ID-11 coefficient block.
    pub f2_pressure_coefficients: [f64; 3],
    /// Two preserved f32 slots from the second ID-11 power block.
    pub f2_pressure_powers: [f32; 2],
    /// Five preserved f64 slots from the third ID-11 coefficient block.
    pub f3_temperature_coefficients: [f64; 5],
    /// Three preserved f32 slots from the third ID-11 power block.
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
