#![no_main]

use factsage_compound_parser::{DatabaseEditor, RawChunk};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 1_048_576 {
        return;
    }
    if let Ok(mut editor) = DatabaseEditor::from_bytes(data) {
        let chunk_count = editor.raw().chunks().len();
        let index = data.first().map_or(0, |byte| usize::from(*byte)) % (chunk_count + 1);
        let _ = editor.insert_chunk(
            index,
            RawChunk::Unknown {
                id: 250,
                body: [0; 255],
            },
        );
        let _ = editor.view();
        let _ = editor.to_bytes();
    }
});
