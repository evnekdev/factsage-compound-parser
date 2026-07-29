use super::Cursor;
use crate::error::ParseError;

/// The 31-byte header shared by every non-database CDB body.
#[derive(Debug, Clone, PartialEq)]
pub struct RawCommonHeader {
    /// Seven element identifiers.
    pub element_ids: [u8; 7],
    /// The coefficient padding byte.
    pub coefficient_padding: u8,
    /// Seven integer element coefficients.
    pub element_coefficients: [u8; 7],
    /// The signed raw formula charge.
    pub charge_raw: i8,
    /// The source entry number.
    pub entry_number: u8,
    /// The two raw reference values.
    pub reference: [u16; 2],
    /// The OLE timestamp represented as its raw little-endian f64 value.
    pub timestamp_ole: f64,
    /// Two uninterpreted bytes following the timestamp.
    pub unknown: [u8; 2],
}

pub(crate) fn parse(cursor: &mut Cursor<'_>) -> Result<RawCommonHeader, ParseError> {
    Ok(RawCommonHeader {
        element_ids: cursor.read_u8_array("element_ids")?,
        coefficient_padding: cursor.read_u8("coefficient_padding")?,
        element_coefficients: cursor.read_u8_array("element_coefficients")?,
        charge_raw: cursor.read_i8("charge_raw")?,
        entry_number: cursor.read_u8("entry_number")?,
        reference: cursor.read_u16_array("reference")?,
        timestamp_ole: cursor.read_f64("timestamp_ole")?,
        unknown: cursor.read_u8_array("unknown")?,
    })
}
