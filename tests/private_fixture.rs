use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use factsage_compound_parser::{RawChunk, RawDatabase};

#[test]
fn parses_private_ms16base_when_available() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("MS16BASE.CDB");
    if !path.is_file() {
        return;
    }

    let bytes = fs::read(&path).expect("private fixture should be readable");
    assert_eq!(bytes.len(), 755_712);

    let database = RawDatabase::from_bytes(&bytes).expect("private fixture should parse");
    assert_eq!(database.chunks.len(), 2_952);

    match &database.chunks[0] {
        RawChunk::DatabaseHeader(header) => assert_eq!(header.magic, *b"CMPD"),
        _ => panic!("private fixture must begin with database header"),
    }

    let mut histogram = BTreeMap::new();
    for chunk in &database.chunks {
        *histogram.entry(chunk.id()).or_insert(0_usize) += 1;
    }

    let expected = BTreeMap::from([
        (1_u8, 537_usize),
        (2_u8, 1_517_usize),
        (5_u8, 1_usize),
        (7_u8, 620_usize),
        (8_u8, 64_usize),
        (9_u8, 1_usize),
        (10_u8, 212_usize),
    ]);
    assert_eq!(histogram, expected);
}
