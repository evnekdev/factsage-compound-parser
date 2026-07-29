use factsage_compound_parser::{
    BODY_SIZE, CHUNK_SIZE, HeatCapacityKind, ParseError, RawChunk, RawDatabase,
};

fn set_bytes(chunk: &mut [u8], offset: usize, bytes: &[u8]) {
    chunk[offset..offset + bytes.len()].copy_from_slice(bytes);
}

fn write_u16(chunk: &mut [u8], offset: usize, value: u16) {
    set_bytes(chunk, offset, &value.to_le_bytes());
}

fn write_i32(chunk: &mut [u8], offset: usize, value: i32) {
    set_bytes(chunk, offset, &value.to_le_bytes());
}

fn write_u32(chunk: &mut [u8], offset: usize, value: u32) {
    set_bytes(chunk, offset, &value.to_le_bytes());
}

fn write_f32(chunk: &mut [u8], offset: usize, value: f32) {
    set_bytes(chunk, offset, &value.to_le_bytes());
}

fn write_f64(chunk: &mut [u8], offset: usize, value: f64) {
    set_bytes(chunk, offset, &value.to_le_bytes());
}

fn fill_common_header(chunk: &mut [u8], charge: i8, timestamp: f64) {
    for index in 0..7 {
        chunk[1 + index] = index as u8 + 1;
        chunk[9 + index] = index as u8 + 11;
    }
    chunk[8] = 0xa5;
    chunk[16] = charge as u8;
    chunk[17] = 7;
    write_u16(chunk, 18, 0x1234);
    write_u16(chunk, 20, 0xabcd);
    write_f64(chunk, 22, timestamp);
    chunk[30] = 0x5a;
    chunk[31] = 0xa5;
}

fn database_header_chunk() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 9;
    chunk[1] = 0x7f;
    set_bytes(&mut chunk, 2, b"CMPD");
    chunk[6] = 0x11;
    chunk[7] = 0x22;
    write_f64(&mut chunk, 8, 44_398.5);
    chunk[16] = 1;
    for byte in &mut chunk[17..28] {
        *byte = 0x33;
    }
    set_bytes(&mut chunk, 28, b"synthetic header");
    for byte in &mut chunk[108..244] {
        *byte = 0x44;
    }
    for byte in &mut chunk[244..256] {
        *byte = 0x55;
    }
    chunk
}

fn compound_chunk() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 1;
    fill_common_header(&mut chunk, -7, 12.5);
    set_bytes(&mut chunk, 32, b"Synthetic compound");
    set_bytes(&mut chunk, 72, b"reserved");
    set_bytes(&mut chunk, 112, b"X2Y");
    set_bytes(&mut chunk, 152, &[0xde, 0xad, 0xbe, 0xef]);
    write_u32(&mut chunk, 156, 0x1122_3344);
    write_u32(&mut chunk, 160, 0x5566_7788);
    set_bytes(&mut chunk, 164, b"unit-reserved");
    for index in 0..7 {
        write_f64(&mut chunk, 176 + index * 8, index as f64 + 0.5);
    }
    chunk
}

fn ordinary_phase_chunk() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 7;
    fill_common_header(&mut chunk, -3, 1.25);
    write_f64(&mut chunk, 32, -123.5);
    write_f64(&mut chunk, 40, 45.25);
    write_i32(&mut chunk, 48, -800);
    write_i32(&mut chunk, 52, -801);
    write_f64(&mut chunk, 56, 7.5);
    for index in 0..4 {
        write_f32(&mut chunk, 64 + index * 4, index as f32 + 0.25);
        write_f32(&mut chunk, 80 + index * 4, index as f32 + 1.25);
    }
    for index in 0..2 {
        write_f32(&mut chunk, 96 + index * 4, index as f32 + 2.25);
    }
    write_f32(&mut chunk, 104, 1000.0);
    write_f32(&mut chunk, 108, 2.0);
    write_f32(&mut chunk, 112, 3.0);
    set_bytes(&mut chunk, 136, b"synthetic ordinary");
    chunk
}

fn transition_phase_chunk() -> [u8; CHUNK_SIZE] {
    let mut chunk = ordinary_phase_chunk();
    chunk[0] = 8;
    write_f64(&mut chunk, 32, 999.0);
    write_f64(&mut chunk, 40, 1200.0);
    write_i32(&mut chunk, 48, 101);
    write_i32(&mut chunk, 52, 801);
    chunk
}

fn heat_capacity_chunk(id: u8) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = id;
    fill_common_header(&mut chunk, 4, 2.5);
    write_f64(&mut chunk, 32, 10.0);
    write_f64(&mut chunk, 40, 20.0);
    write_i32(&mut chunk, 48, 801);
    set_bytes(&mut chunk, 52, &[1, 2, 3, 4]);
    write_f64(&mut chunk, 56, 298.15);
    write_f64(&mut chunk, 64, 1000.0);
    for index in 0..8 {
        write_f64(&mut chunk, 72 + index * 8, index as f64 + 0.5);
        write_f64(&mut chunk, 136 + index * 8, index as f64 - 2.0);
    }
    chunk
}

fn comment_chunk() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 10;
    fill_common_header(&mut chunk, 0, 3.5);
    set_bytes(&mut chunk, 32, b"degree ");
    chunk[39] = 0xb0;
    set_bytes(&mut chunk, 40, b" Celsius");
    chunk
}

fn kappa_chunk() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 11;
    fill_common_header(&mut chunk, 1, 4.5);
    write_f64(&mut chunk, 32, 300.0);
    write_f64(&mut chunk, 40, 1200.0);
    write_i32(&mut chunk, 48, 901);
    set_bytes(&mut chunk, 52, &[9, 8, 7, 6]);
    for index in 0..10 {
        write_f64(&mut chunk, 56 + index * 8, index as f64);
    }
    for index in 0..8 {
        write_f32(&mut chunk, 136 + index * 4, index as f32);
    }
    for index in 0..3 {
        write_f64(&mut chunk, 168 + index * 8, index as f64 + 10.0);
    }
    for index in 0..2 {
        write_f32(&mut chunk, 192 + index * 4, index as f32 + 20.0);
    }
    for index in 0..5 {
        write_f64(&mut chunk, 200 + index * 8, index as f64 + 30.0);
    }
    for index in 0..3 {
        write_f32(&mut chunk, 240 + index * 4, index as f32 + 40.0);
    }
    chunk
}

fn unknown_chunk() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 200;
    for (index, byte) in chunk[1..].iter_mut().enumerate() {
        *byte = (index as u8).wrapping_mul(3);
    }
    chunk
}

fn database_with(chunks: &[[u8; CHUNK_SIZE]]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((chunks.len() + 1) * CHUNK_SIZE);
    bytes.extend_from_slice(&database_header_chunk());
    for chunk in chunks {
        bytes.extend_from_slice(chunk);
    }
    bytes
}

#[test]
fn parses_valid_database_header() {
    let database = RawDatabase::from_bytes(&database_header_chunk()).unwrap();

    assert_eq!(database.chunks.len(), 1);
    match &database.chunks[0] {
        RawChunk::DatabaseHeader(header) => {
            assert_eq!(header.magic, *b"CMPD");
            assert_eq!(header.comment_lossy(), "synthetic header");
            assert_eq!(header.padding_3.len(), 136);
        }
        other => panic!("unexpected chunk: {:?}", other.id()),
    }
}

#[test]
fn rejects_invalid_magic() {
    let mut chunk = database_header_chunk();
    chunk[2..6].copy_from_slice(b"FAIL");

    let error = RawDatabase::from_bytes(&chunk).unwrap_err();
    assert!(matches!(
        error,
        ParseError::InvalidDatabaseMagic {
            chunk_index: 0,
            byte_offset: 2,
            ..
        }
    ));
}

#[test]
fn rejects_empty_and_misaligned_files() {
    assert!(matches!(
        RawDatabase::from_bytes(&[]),
        Err(ParseError::EmptyFile)
    ));
    assert!(matches!(
        RawDatabase::from_bytes(&[0; 255]),
        Err(ParseError::InvalidFileLength {
            length: 255,
            chunk_size: 256
        })
    ));
}

#[test]
fn rejects_invalid_first_chunk_id() {
    let mut chunk = database_header_chunk();
    chunk[0] = 1;

    let error = RawDatabase::from_bytes(&chunk).unwrap_err();
    assert!(matches!(
        error,
        ParseError::InvalidFirstChunkId {
            found: 1,
            expected: 9
        }
    ));
}

#[test]
fn parses_compound_and_little_endian_fields() {
    let compound = compound_chunk();
    let database = RawDatabase::from_bytes(&database_with(&[compound])).unwrap();

    match &database.chunks[1] {
        RawChunk::Compound(value) => {
            assert_eq!(value.header.charge_raw, -7);
            assert_eq!(value.unit_energy, 0x1122_3344);
            assert_eq!(value.unit_pressure, 0x5566_7788);
            assert_eq!(value.compound_name_lossy(), "Synthetic compound");
            assert_eq!(value.formula_name_lossy(), "X2Y");
            assert_eq!(value.real_stoichiometric_coefficients.len(), 7);
            assert_eq!(value.unknown, [0xde, 0xad, 0xbe, 0xef]);
        }
        _ => panic!("expected compound"),
    }
}

#[test]
fn parses_ordinary_and_transition_phases_with_signed_ids() {
    let database = RawDatabase::from_bytes(&database_with(&[
        ordinary_phase_chunk(),
        transition_phase_chunk(),
    ]))
    .unwrap();

    match &database.chunks[1] {
        RawChunk::PhaseOrdinary(value) => {
            assert_eq!(value.phase_id_raw_neg, -800);
            assert_eq!(value.phase_id_raw, -801);
            assert_eq!(value.physical.phase_name_lossy(), "synthetic ordinary");
            assert_eq!(value.physical.thermal_expansion_coefficients.len(), 4);
        }
        _ => panic!("expected ordinary phase"),
    }
    match &database.chunks[2] {
        RawChunk::PhaseTransition(value) => {
            assert_eq!(value.parent_phase_id_raw, 101);
            assert_eq!(value.phase_id_raw, 801);
            assert_eq!(value.transition_temperature, 1200.0);
            assert_eq!(value.physical.padding_2.len(), 80);
        }
        _ => panic!("expected transition phase"),
    }
}

#[test]
fn parses_cp_chunk_and_preserves_all_ids() {
    let database = RawDatabase::from_bytes(&database_with(&[heat_capacity_chunk(2)])).unwrap();

    match &database.chunks[1] {
        RawChunk::HeatCapacity { kind, chunk } => {
            assert_eq!(*kind, HeatCapacityKind::Id2);
            assert_eq!(kind.id(), 2);
            assert_eq!(chunk.temperature_min, 298.15);
            assert_eq!(chunk.coefficients.len(), 8);
            assert_eq!(chunk.powers.len(), 8);
            assert_eq!(chunk.padding_remaining.len(), 56);
        }
        _ => panic!("expected heat-capacity chunk"),
    }
}

#[test]
fn parses_each_heat_capacity_id_without_collapsing_it() {
    let chunks = [
        heat_capacity_chunk(2),
        heat_capacity_chunk(3),
        heat_capacity_chunk(4),
        heat_capacity_chunk(5),
        heat_capacity_chunk(6),
    ];
    let database = RawDatabase::from_bytes(&database_with(&chunks)).unwrap();

    let ids = database
        .chunks
        .iter()
        .skip(1)
        .map(RawChunk::id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![2, 3, 4, 5, 6]);
}

#[test]
fn parses_windows_1252_comment_without_lossy_binary_parsing() {
    let database = RawDatabase::from_bytes(&database_with(&[comment_chunk()])).unwrap();

    match &database.chunks[1] {
        RawChunk::Comment(value) => {
            assert_eq!(value.comment[7], 0xb0);
            assert_eq!(value.comment_windows_1252(), "degree \u{00b0} Celsius");
            assert_eq!(value.header.charge_raw, 0);
        }
        _ => panic!("expected comment"),
    }
}

#[test]
fn parses_kappa_arrays_with_exact_lengths() {
    let database = RawDatabase::from_bytes(&database_with(&[kappa_chunk()])).unwrap();

    match &database.chunks[1] {
        RawChunk::Kappa(value) => {
            assert_eq!(value.f1_temperature_coefficients.len(), 10);
            assert_eq!(value.f1_temperature_powers.len(), 8);
            assert_eq!(value.f2_pressure_coefficients.len(), 3);
            assert_eq!(value.f2_pressure_powers.len(), 2);
            assert_eq!(value.f3_temperature_coefficients.len(), 5);
            assert_eq!(value.f3_temperature_powers.len(), 3);
            assert_eq!(value.padding_remaining.len(), 4);
            assert_eq!(value.temperature_min, 300.0);
            assert_eq!(value.phase_id_raw, 901);
        }
        _ => panic!("expected kappa"),
    }
}

#[test]
fn preserves_unknown_chunk_body() {
    let unknown = unknown_chunk();
    let database = RawDatabase::from_bytes(&database_with(&[unknown])).unwrap();

    match &database.chunks[1] {
        RawChunk::Unknown { id, body } => {
            assert_eq!(*id, 200);
            assert_eq!(body[0], 0);
            assert_eq!(body[254], (254_u8).wrapping_mul(3));
            assert_eq!(body.len(), BODY_SIZE);
        }
        _ => panic!("expected unknown chunk"),
    }
}

#[test]
fn parses_from_reader() {
    let bytes = database_with(&[compound_chunk()]);
    let database = RawDatabase::from_reader(std::io::Cursor::new(bytes)).unwrap();
    assert_eq!(database.chunks.len(), 2);
}
