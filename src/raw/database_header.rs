use super::Cursor;
use crate::error::ParseError;

/// The ID-9 database-header body.
#[derive(Debug, Clone, PartialEq)]
pub struct RawDatabaseHeaderChunk {
    /// The first padding byte.
    pub padding_1: u8,
    /// The validated four-byte database magic.
    pub magic: [u8; 4],
    /// Two padding bytes following the magic.
    pub padding_2: [u8; 2],
    /// The raw OLE database date.
    pub date_ole: f64,
    /// The database read flag as stored.
    pub read_flag: u8,
    /// Eleven uninterpreted bytes.
    pub unknown_1: [u8; 11],
    /// The fixed-width database comment bytes.
    pub comment: [u8; 80],
    /// Database-header padding bytes.
    pub padding_3: [u8; 136],
    /// Twelve trailing uninterpreted bytes.
    pub unknown_2: [u8; 12],
}

pub(crate) fn parse(
    cursor: &mut Cursor<'_>,
    chunk_index: usize,
) -> Result<RawDatabaseHeaderChunk, ParseError> {
    let padding_1 = cursor.read_u8("padding_1")?;
    let magic_offset = cursor.offset();
    let magic = cursor.read_u8_array("magic")?;
    if magic != *b"CMPD" {
        return Err(ParseError::InvalidDatabaseMagic {
            chunk_index,
            byte_offset: magic_offset,
            found: magic,
        });
    }

    let header = RawDatabaseHeaderChunk {
        padding_1,
        magic,
        padding_2: cursor.read_u8_array("padding_2")?,
        date_ole: cursor.read_f64("date_ole")?,
        read_flag: cursor.read_u8("read_flag")?,
        unknown_1: cursor.read_u8_array("unknown_1")?,
        comment: cursor.read_u8_array("comment")?,
        padding_3: cursor.read_u8_array("padding_3")?,
        unknown_2: cursor.read_u8_array("unknown_2")?,
    };
    cursor.finish()?;
    Ok(header)
}

impl RawDatabaseHeaderChunk {
    /// Returns the fixed-width header comment with NUL/space padding removed.
    pub fn comment_lossy(&self) -> String {
        super::decode_ascii_lossy(&self.comment)
    }
}
