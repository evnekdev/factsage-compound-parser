#![no_main]

use factsage_compound_parser::RawDatabase;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 1_048_576 {
        return;
    }
    if let Ok(raw) = RawDatabase::from_bytes(data) {
        if let Ok(output) = raw.to_bytes() {
            assert_eq!(output, data);
        }
    }
});
