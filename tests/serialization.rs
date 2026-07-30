use factsage_compound_parser::{RawChunk, RawDatabase};

const CHUNK_SIZE: usize = 256;

fn patterned_chunk(id: u8, seed: u8) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = id;
    for (offset, byte) in chunk[1..].iter_mut().enumerate() {
        *byte = seed.wrapping_add((offset as u8).wrapping_mul(37));
    }
    chunk
}

fn header() -> [u8; CHUNK_SIZE] {
    let mut chunk = patterned_chunk(9, 0x91);
    chunk[2..6].copy_from_slice(b"CMPD");
    chunk
}

#[test]
fn serializes_every_chunk_kind_byte_for_byte() {
    let mut chunks = vec![header()];
    for (offset, id) in [1_u8, 2, 3, 4, 5, 6, 7, 8, 10, 11, 250]
        .into_iter()
        .enumerate()
    {
        chunks.push(patterned_chunk(id, 0x11_u8.wrapping_add(offset as u8)));
    }
    let input = chunks.into_iter().flatten().collect::<Vec<_>>();

    let database = RawDatabase::from_bytes(&input).expect("synthetic chunks parse");
    let output = database.to_bytes().expect("synthetic chunks serialize");

    assert_eq!(output, input);
    assert!(matches!(
        database.chunks().last(),
        Some(RawChunk::Unknown { id: 250, .. })
    ));
}

#[test]
fn preserves_nan_payloads_and_infinities_in_typed_chunks() {
    let mut compound = patterned_chunk(1, 0x13);
    compound[176..184].copy_from_slice(&0x7ff8_0000_0000_0042_u64.to_le_bytes());
    compound[184..192].copy_from_slice(&f64::INFINITY.to_le_bytes());

    let mut ordinary = patterned_chunk(7, 0x27);
    ordinary[56..64].copy_from_slice(&0x7ff0_0000_0000_0123_u64.to_le_bytes());
    ordinary[64..68].copy_from_slice(&0x7fc0_0042_u32.to_le_bytes());

    let mut cp = patterned_chunk(6, 0x44);
    cp[72..80].copy_from_slice(&0x7ff8_0000_0000_0abc_u64.to_le_bytes());
    cp[136..144].copy_from_slice(&f64::NEG_INFINITY.to_le_bytes());

    let input = [header(), compound, ordinary, cp]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let database = RawDatabase::from_bytes(&input).expect("synthetic chunks parse");

    assert_eq!(
        database.to_bytes().expect("synthetic chunks serialize"),
        input
    );
}

#[test]
fn writes_the_same_bytes_to_a_generic_writer() {
    let input = [header(), patterned_chunk(10, 0x55)]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let database = RawDatabase::from_bytes(&input).expect("synthetic chunks parse");
    let mut output = Vec::new();

    database
        .write_to(&mut output)
        .expect("vector writer accepts bytes");

    assert_eq!(output, input);
}
