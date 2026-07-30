use std::borrow::Cow;

use encoding_rs::WINDOWS_1252;

use super::error::TextDecodeError;

pub(crate) fn trim_storage_padding(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let mut trimmed_end = end;
    while trimmed_end > 0 && bytes[trimmed_end - 1] == b' ' {
        trimmed_end -= 1;
    }
    &bytes[..trimmed_end]
}

pub(crate) fn decode_ascii<'a>(
    bytes: &'a [u8],
    field: &'static str,
) -> Result<Cow<'a, str>, TextDecodeError> {
    let trimmed = trim_storage_padding(bytes);
    if let Some((index, _)) = trimmed.iter().enumerate().find(|(_, byte)| **byte > 0x7f) {
        return Err(TextDecodeError::new(field, index));
    }

    match std::str::from_utf8(trimmed) {
        Ok(text) => Ok(Cow::Borrowed(text)),
        Err(error) => Err(TextDecodeError::new(field, error.valid_up_to())),
    }
}

pub(crate) fn decode_windows_1252(bytes: &[u8]) -> Cow<'_, str> {
    let trimmed = trim_storage_padding(bytes);
    WINDOWS_1252.decode(trimmed).0
}
