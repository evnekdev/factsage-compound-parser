use super::{Cursor, common_header};
use crate::error::ParseError;

/// An ID-10 fixed-width comment fragment.
#[derive(Debug, Clone, PartialEq)]
pub struct RawCommentChunk {
    /// The shared non-database header.
    pub header: super::RawCommonHeader,
    /// The raw Windows-1252-compatible comment bytes.
    pub comment: [u8; 80],
    /// Remaining comment-body padding.
    pub padding_remaining: [u8; 144],
}

pub(crate) fn parse(cursor: &mut Cursor<'_>) -> Result<RawCommentChunk, ParseError> {
    let chunk = RawCommentChunk {
        header: common_header::parse(cursor)?,
        comment: cursor.read_u8_array("comment")?,
        padding_remaining: cursor.read_u8_array("padding_remaining")?,
    };
    cursor.finish()?;
    Ok(chunk)
}

impl RawCommentChunk {
    /// Decodes the comment as Windows-1252 without panicking on invalid input.
    pub fn comment_windows_1252(&self) -> String {
        super::decode_windows_1252(&self.comment)
    }
}
