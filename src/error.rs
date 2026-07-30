use std::fmt;
use std::io;

/// Errors returned while parsing the fixed-width CDB binary format.
#[derive(Debug)]
pub enum ParseError {
    /// The input contains no chunks.
    EmptyFile,
    /// The input length is not an exact multiple of the fixed chunk size.
    InvalidFileLength {
        /// Total input byte length.
        length: usize,
        /// Required physical record size in bytes.
        chunk_size: usize,
    },
    /// The first chunk is not the required database-header chunk.
    InvalidFirstChunkId {
        /// Observed one-byte chunk ID.
        found: u8,
        /// Required one-byte database-header ID.
        expected: u8,
    },
    /// A database-header body does not contain the required magic bytes.
    InvalidDatabaseMagic {
        /// Zero-based physical chunk index.
        chunk_index: usize,
        /// Absolute byte offset of the magic field.
        byte_offset: usize,
        /// Bytes observed instead of `CMPD`.
        found: [u8; 4],
    },
    /// A chunk was expected to contain 256 bytes but did not.
    TruncatedRecord {
        /// Zero-based chunk index of the partial record.
        chunk_index: usize,
        /// Absolute byte offset at which the record starts.
        byte_offset: usize,
        /// Required record size in bytes.
        expected: usize,
        /// Number of bytes actually supplied for this record.
        actual: usize,
    },
    /// A typed body could not read one of its fixed-width fields.
    FieldBoundary {
        /// Zero-based physical chunk index.
        chunk_index: usize,
        /// Absolute byte offset of the attempted field read.
        byte_offset: usize,
        /// Logical name of the current record layout.
        record_type: &'static str,
        /// Logical field name at the failed boundary.
        field: &'static str,
        /// Number of bytes requested by the field reader.
        requested: usize,
        /// Bytes still available in the fixed record body.
        remaining: usize,
    },
    /// The parser could not reserve owned storage for the requested raw records.
    Allocation {
        /// Number of raw chunks whose storage was being reserved.
        chunk_count: usize,
    },
    /// The input contains more records than a `usize` counter can represent.
    RecordCountOverflow,
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
            Self::Allocation { chunk_count } => write!(
                formatter,
                "could not reserve owned storage for {chunk_count} CDB chunk(s)"
            ),
            Self::RecordCountOverflow => formatter.write_str("CDB record count overflows usize"),
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
