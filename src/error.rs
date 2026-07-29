use std::fmt;
use std::io;

/// Errors returned while parsing the fixed-width CDB binary format.
#[derive(Debug)]
pub enum ParseError {
    /// The input contains no chunks.
    EmptyFile,
    /// The input length is not an exact multiple of the fixed chunk size.
    InvalidFileLength { length: usize, chunk_size: usize },
    /// The first chunk is not the required database-header chunk.
    InvalidFirstChunkId { found: u8, expected: u8 },
    /// A database-header body does not contain the required magic bytes.
    InvalidDatabaseMagic {
        chunk_index: usize,
        byte_offset: usize,
        found: [u8; 4],
    },
    /// A chunk was expected to contain 256 bytes but did not.
    TruncatedRecord {
        chunk_index: usize,
        byte_offset: usize,
        expected: usize,
        actual: usize,
    },
    /// A typed body could not read one of its fixed-width fields.
    FieldBoundary {
        chunk_index: usize,
        byte_offset: usize,
        record_type: &'static str,
        field: &'static str,
        requested: usize,
        remaining: usize,
    },
    /// Reading the input source failed.
    Io(io::Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => formatter.write_str("CDB file is empty"),
            Self::InvalidFileLength { length, chunk_size } => write!(
                formatter,
                "CDB file length {} is not divisible by chunk size {}",
                length, chunk_size
            ),
            Self::InvalidFirstChunkId { found, expected } => write!(
                formatter,
                "first CDB chunk has ID {}, expected {}",
                found, expected
            ),
            Self::InvalidDatabaseMagic {
                chunk_index,
                byte_offset,
                found,
            } => write!(
                formatter,
                "invalid database magic at chunk {} byte offset {}: found {:?}, expected CMPD",
                chunk_index, byte_offset, found
            ),
            Self::TruncatedRecord {
                chunk_index,
                byte_offset,
                expected,
                actual,
            } => write!(
                formatter,
                "truncated record at chunk {} byte offset {}: expected {} bytes, got {}",
                chunk_index, byte_offset, expected, actual
            ),
            Self::FieldBoundary {
                chunk_index,
                byte_offset,
                record_type,
                field,
                requested,
                remaining,
            } => write!(
                formatter,
                "field boundary error at chunk {} byte offset {} in {}.{}: requested {} bytes with {} remaining",
                chunk_index, byte_offset, record_type, field, requested, remaining
            ),
            Self::Io(error) => write!(formatter, "I/O error while reading CDB: {}", error),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
