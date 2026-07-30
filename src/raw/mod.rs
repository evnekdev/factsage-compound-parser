pub mod chunk;
pub mod comment;
pub mod common_header;
pub mod compound;
mod cursor;
pub mod database_header;
pub mod heat_capacity;
pub mod kappa;
pub mod phase;
mod serialize;

pub use chunk::RawChunk;
pub use comment::RawCommentChunk;
pub use common_header::RawCommonHeader;
pub use compound::RawCompoundChunk;
pub use database_header::RawDatabaseHeaderChunk;
pub use heat_capacity::{HeatCapacityKind, RawHeatCapacityChunk};
pub use kappa::RawKappaChunk;
pub use phase::{RawOrdinaryPhaseChunk, RawPhasePhysicalTail, RawTransitionPhaseChunk};
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
