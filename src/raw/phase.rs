use super::{Cursor, common_header};
use crate::error::ParseError;

/// The common 200-byte physical-property tail of phase records.
#[derive(Debug, Clone, PartialEq)]
pub struct RawPhasePhysicalTail {
    /// The raw density value.
    pub density_raw: f64,
    /// Four thermal-expansion coefficients.
    pub thermal_expansion_coefficients: [f32; 4],
    /// Four compressibility coefficients.
    pub compressibility_coefficients: [f32; 4],
    /// Two bulk-modulus derivative coefficients.
    pub bulk_modulus_derivative_coefficients: [f32; 2],
    /// The raw magnetic temperature.
    pub magnetic_temperature: f32,
    /// The raw magnetic moment.
    pub magnetic_moment: f32,
    /// The raw p-factor.
    pub p_factor: f32,
    /// Twenty uninterpreted padding bytes.
    pub padding_1: [u8; 20],
    /// The fixed-width phase name bytes.
    pub phase_name: [u8; 40],
    /// Eighty trailing padding bytes.
    pub padding_2: [u8; 80],
}

pub(crate) fn parse_physical_tail(
    cursor: &mut Cursor<'_>,
) -> Result<RawPhasePhysicalTail, ParseError> {
    Ok(RawPhasePhysicalTail {
        density_raw: cursor.read_f64("density_raw")?,
        thermal_expansion_coefficients: cursor.read_f32_array("thermal_expansion_coefficients")?,
        compressibility_coefficients: cursor.read_f32_array("compressibility_coefficients")?,
        bulk_modulus_derivative_coefficients: cursor
            .read_f32_array("bulk_modulus_derivative_coefficients")?,
        magnetic_temperature: cursor.read_f32("magnetic_temperature")?,
        magnetic_moment: cursor.read_f32("magnetic_moment")?,
        p_factor: cursor.read_f32("p_factor")?,
        padding_1: cursor.read_u8_array("padding_1")?,
        phase_name: cursor.read_u8_array("phase_name")?,
        padding_2: cursor.read_u8_array("padding_2")?,
    })
}

impl RawPhasePhysicalTail {
    /// Returns the phase name with fixed-width padding removed.
    pub fn phase_name_lossy(&self) -> String {
        super::decode_ascii_lossy(&self.phase_name)
    }
}

/// An ID-7 ordinary phase body.
#[derive(Debug, Clone, PartialEq)]
pub struct RawOrdinaryPhaseChunk {
    /// The shared non-database header.
    pub header: super::RawCommonHeader,
    /// The ordinary phase enthalpy.
    pub enthalpy: f64,
    /// The ordinary phase entropy.
    pub entropy: f64,
    /// The first signed raw phase identifier.
    pub phase_id_raw_neg: i32,
    /// The phase identifier.
    pub phase_id_raw: i32,
    /// The common physical-property tail.
    pub physical: RawPhasePhysicalTail,
}

pub(crate) fn parse_ordinary(cursor: &mut Cursor<'_>) -> Result<RawOrdinaryPhaseChunk, ParseError> {
    let chunk = RawOrdinaryPhaseChunk {
        header: common_header::parse(cursor)?,
        enthalpy: cursor.read_f64("enthalpy")?,
        entropy: cursor.read_f64("entropy")?,
        phase_id_raw_neg: cursor.read_i32("phase_id_raw_neg")?,
        phase_id_raw: cursor.read_i32("phase_id_raw")?,
        physical: parse_physical_tail(cursor)?,
    };
    cursor.finish()?;
    Ok(chunk)
}

/// An ID-8 transition phase body.
#[derive(Debug, Clone, PartialEq)]
pub struct RawTransitionPhaseChunk {
    /// The shared non-database header.
    pub header: super::RawCommonHeader,
    /// The transition enthalpy.
    pub transition_enthalpy: f64,
    /// The transition temperature.
    pub transition_temperature: f64,
    /// The parent phase identifier.
    pub parent_phase_id_raw: i32,
    /// The transition phase identifier.
    pub phase_id_raw: i32,
    /// The common physical-property tail.
    pub physical: RawPhasePhysicalTail,
}

pub(crate) fn parse_transition(
    cursor: &mut Cursor<'_>,
) -> Result<RawTransitionPhaseChunk, ParseError> {
    let chunk = RawTransitionPhaseChunk {
        header: common_header::parse(cursor)?,
        transition_enthalpy: cursor.read_f64("transition_enthalpy")?,
        transition_temperature: cursor.read_f64("transition_temperature")?,
        parent_phase_id_raw: cursor.read_i32("parent_phase_id_raw")?,
        phase_id_raw: cursor.read_i32("phase_id_raw")?,
        physical: parse_physical_tail(cursor)?,
    };
    cursor.finish()?;
    Ok(chunk)
}
