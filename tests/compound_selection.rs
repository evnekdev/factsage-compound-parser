use factsage_compound_parser::{CHUNK_SIZE, Database, edit::CompoundSelectionError};

fn chunk(id: u8) -> [u8; CHUNK_SIZE] {
    let mut bytes = [0; CHUNK_SIZE];
    bytes[0] = id;
    if id == 9 {
        bytes[2..6].copy_from_slice(b"CMPD");
    }
    bytes
}

#[test]
fn retention_preserves_preamble_unknown_bytes_order_and_source() {
    let mut opaque = chunk(200);
    opaque[12] = 73;
    let input = [
        chunk(9),
        chunk(1),
        chunk(7),
        chunk(1),
        chunk(7),
        opaque,
        chunk(1),
        chunk(7),
    ]
    .concat();
    let database = Database::from_bytes(&input).unwrap();
    let retained = database.retain_compound_groups(&[1, 1]).unwrap();
    let expected = [chunk(9), chunk(1), chunk(7), opaque].concat();
    assert_eq!(retained.raw().to_bytes().unwrap(), expected);
    assert_eq!(database.raw().to_bytes().unwrap(), input);
    let reparsed = Database::from_bytes(&expected).unwrap();
    assert_eq!(reparsed.view().unwrap().compound_count(), 1);
    assert_eq!(
        database
            .retain_compound_groups(&[])
            .unwrap()
            .raw()
            .to_bytes()
            .unwrap(),
        chunk(9)
    );
    assert!(matches!(
        database.retain_compound_groups(&[3]),
        Err(CompoundSelectionError::UnknownCompound { index: 3, count: 3 })
    ));
}
