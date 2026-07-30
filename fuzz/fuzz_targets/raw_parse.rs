#![no_main]

use factsage_compound_parser::RawDatabase;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 1_048_576 {
        return;
    }
    let _ = RawDatabase::from_bytes(data);
    let _ = RawDatabase::from_reader(std::io::Cursor::new(data));
});
