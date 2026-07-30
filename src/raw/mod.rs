/// Typed raw CDB chunk variants and their parser.
pub mod chunk;
/// Raw fixed-width comment fragments.
pub mod comment;
/// The shared entry header used by non-database chunks.
pub mod common_header;
/// Raw compound chunks.
pub mod compound;
mod cursor;
/// Raw database-header chunks.
pub mod database_header;
/// Errors from low-level raw structural edits.
pub mod error;
/// Raw heat-capacity chunks and their preserved IDs.
pub mod heat_capacity;
/// Raw kappa chunks.
pub mod kappa;
/// Raw ordinary and transition phase chunks.
pub mod phase;
mod serialize;

/// Re-export of all typed raw chunk variants.
pub use chunk::RawChunk;
/// Re-export of raw comment fragments.
pub use comment::RawCommentChunk;
/// Re-export of the shared entry header.
pub use common_header::RawCommonHeader;
/// Re-export of raw compound chunks.
pub use compound::RawCompoundChunk;
/// Re-export of raw database-header chunks.
pub use database_header::RawDatabaseHeaderChunk;
/// Re-export of raw structural edit errors.
pub use error::RawEditError;
/// Re-export of heat-capacity IDs and chunks.
pub use heat_capacity::{HeatCapacityKind, RawHeatCapacityChunk};
/// Re-export of raw kappa chunks.
pub use kappa::RawKappaChunk;
/// Re-export of raw phase chunks and their common physical tail.
pub use phase::{RawOrdinaryPhaseChunk, RawPhasePhysicalTail, RawTransitionPhaseChunk};
/// Re-export of raw serialization errors.
pub use serialize::SerializeError;

/// The flat lossless raw CDB stream. Re-exported here for module-oriented imports.
pub use crate::RawDatabase;

pub(crate) use cursor::Cursor;

fn trim_fixed_bytes(bytes: &[u8]) -> &[u8] {
    let without_nul = match bytes.iter().position(|byte| *byte == 0) {
        Some(index) => &bytes[..index],
        None => bytes,
    };
    let mut end = without_nul.len();
    while end > 0 && without_nul[end - 1] == b' ' {
        end -= 1;
    }
    &without_nul[..end]
}

pub(crate) fn decode_ascii_lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(trim_fixed_bytes(bytes)).into_owned()
}

pub(crate) fn decode_windows_1252(bytes: &[u8]) -> String {
    let (text, _, _) = encoding_rs::WINDOWS_1252.decode(trim_fixed_bytes(bytes));
    text.into_owned()
}
