use factsage_compound_parser::{
    Database, DatabaseError, DiagnosticKind, DomainError, HeatCapacityKind, PhaseState,
};

const CHUNK_SIZE: usize = 256;

fn put<const N: usize>(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: [u8; N]) {
    chunk[offset..offset + N].copy_from_slice(&value);
}

fn put_ascii(chunk: &mut [u8; CHUNK_SIZE], offset: usize, text: &[u8]) {
    let length = text.len().min(CHUNK_SIZE - offset);
    chunk[offset..offset + length].copy_from_slice(&text[..length]);
}

fn put_i32(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: i32) {
    put(chunk, offset, value.to_le_bytes());
}

fn put_f64(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: f64) {
    put(chunk, offset, value.to_le_bytes());
}

fn common(chunk: &mut [u8; CHUNK_SIZE]) {
    chunk[1..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
    chunk[8] = 0xaa;
    chunk[9..16].copy_from_slice(&[8, 9, 10, 11, 12, 13, 14]);
    chunk[16] = (-2_i8) as u8;
    chunk[17] = 42;
    put(chunk, 18, 0x1234_u16.to_le_bytes());
    put(chunk, 20, 0x5678_u16.to_le_bytes());
    put_f64(chunk, 22, 123.5);
    chunk[30..32].copy_from_slice(&[0xbb, 0xcc]);
}

fn header() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 9;
    chunk[2..6].copy_from_slice(b"CMPD");
    chunk
}

fn compound(name: &str, formula: &str) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 1;
    common(&mut chunk);
    put_ascii(&mut chunk, 32, name.as_bytes());
    put_ascii(&mut chunk, 112, formula.as_bytes());
    for index in 0..7 {
        put_f64(&mut chunk, 176 + index * 8, (index + 1) as f64);
    }
    chunk
}

fn ordinary(phase_id_raw: i32, name: &str) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 7;
    common(&mut chunk);
    put_f64(&mut chunk, 32, 10.5);
    put_f64(&mut chunk, 40, 20.5);
    put_i32(&mut chunk, 48, -phase_id_raw);
    put_i32(&mut chunk, 52, phase_id_raw);
    put_ascii(&mut chunk, 136, name.as_bytes());
    chunk
}

fn transition(phase_id_raw: i32, parent_phase_id_raw: i32, name: &str) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 8;
    common(&mut chunk);
    put_f64(&mut chunk, 32, 30.5);
    put_f64(&mut chunk, 40, 900.0);
    put_i32(&mut chunk, 48, parent_phase_id_raw);
    put_i32(&mut chunk, 52, phase_id_raw);
    put_ascii(&mut chunk, 136, name.as_bytes());
    chunk
}

fn heat_capacity(id: u8, phase_id_raw: i32, t_min: f64, t_max: f64) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = id;
    common(&mut chunk);
    put_f64(&mut chunk, 32, 1.0);
    put_f64(&mut chunk, 40, 2.0);
    put_i32(&mut chunk, 48, phase_id_raw);
    put_f64(&mut chunk, 56, t_min);
    put_f64(&mut chunk, 64, t_max);
    chunk
}

fn kappa(phase_id_raw: i32, t_min: f64, t_max: f64) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 11;
    common(&mut chunk);
    put_f64(&mut chunk, 32, t_min);
    put_f64(&mut chunk, 40, t_max);
    put_i32(&mut chunk, 48, phase_id_raw);
    chunk
}

fn comment(bytes: &[u8]) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 10;
    common(&mut chunk);
    put_ascii(&mut chunk, 32, bytes);
    chunk
}

fn unknown(id: u8) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = id;
    for (index, value) in chunk[1..].iter_mut().enumerate() {
        *value = index as u8;
    }
    chunk
}

fn database(chunks: impl IntoIterator<Item = [u8; CHUNK_SIZE]>) -> Database {
    let bytes = chunks.into_iter().flatten().collect::<Vec<_>>();
    Database::from_bytes(&bytes).expect("synthetic database should parse")
}

#[test]
fn header_plus_empty_compound() {
    let database = database([header(), compound("Water", "H2O")]);
    assert_eq!(database.compounds.len(), 1);
    assert!(database.compounds[0].phases.is_empty());
    assert!(database.diagnostics.is_empty());
}

#[test]
fn groups_multiple_ordinary_phases() {
    let database = database([
        header(),
        compound("Water", "H2O"),
        ordinary(101, "alpha"),
        ordinary(102, "beta"),
    ]);
    assert_eq!(database.compounds[0].phases.len(), 2);
    assert_eq!(database.compounds[0].phases[1].phase_id_raw(), 102);
}

#[test]
fn groups_ordinary_and_transition_phases() {
    let database = database([
        header(),
        compound("Alloy", "Fe"),
        ordinary(101, "solid"),
        transition(201, 101, "transition"),
    ]);
    assert!(!database.compounds[0].phases[0].is_transition());
    assert!(database.compounds[0].phases[1].is_transition());
}

#[test]
fn attaches_cp_by_raw_phase_id() {
    let database = database([
        header(),
        compound("Water", "H2O"),
        ordinary(101, "solid"),
        heat_capacity(2, 101, 298.0, 1000.0),
    ]);
    let phase = &database.compounds[0].phases[0];
    assert_eq!(phase.heat_capacity_ranges.len(), 1);
    assert_eq!(phase.heat_capacity_ranges[0].kind, HeatCapacityKind::Id2);
    assert!(database.compounds[0].orphan_ranges.is_empty());
}

#[test]
fn attaches_kappa_by_raw_phase_id() {
    let database = database([
        header(),
        compound("Water", "H2O"),
        ordinary(101, "solid"),
        kappa(101, 298.0, 1000.0),
    ]);
    assert_eq!(
        database.compounds[0].phases[0]
            .physical_property_ranges
            .len(),
        1
    );
}

#[test]
fn retains_multiple_cp_ids_after_attachment() {
    let database = database([
        header(),
        compound("Water", "H2O"),
        ordinary(101, "solid"),
        heat_capacity(2, 101, 298.0, 500.0),
        heat_capacity(5, 101, 500.0, 1000.0),
    ]);
    let kinds = database.compounds[0].phases[0]
        .heat_capacity_ranges
        .iter()
        .map(|range| range.kind)
        .collect::<Vec<_>>();
    assert_eq!(kinds, vec![HeatCapacityKind::Id2, HeatCapacityKind::Id5]);
}

#[test]
fn joins_comment_fragments_without_separators() {
    let database = database([
        header(),
        compound("Water", "H2O"),
        comment(b"hello"),
        comment(&[0xb0, b' ']),
        comment(b"world"),
    ]);
    assert_eq!(database.compounds[0].joined_comment(), "hello°world");
}

#[test]
fn duplicate_phase_ids_are_ambiguous_and_preserved() {
    let database = database([
        header(),
        compound("Duplicate", "X"),
        ordinary(101, "one"),
        ordinary(101, "two"),
        heat_capacity(2, 101, 298.0, 1000.0),
    ]);
    assert_eq!(database.compounds[0].phases.len(), 2);
    assert_eq!(database.compounds[0].orphan_ranges.len(), 1);
    assert!(database.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.kind,
        DiagnosticKind::DuplicatePhaseId { phase_id_raw: 101 }
    )));
    assert!(database.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.kind,
        DiagnosticKind::AmbiguousPhaseLink {
            phase_id_raw: 101,
            phase_count: 2
        }
    )));
}

#[test]
fn orphan_cp_is_preserved_and_reported() {
    let database = database([
        header(),
        compound("Orphan", "X"),
        heat_capacity(3, 101, 298.0, 1000.0),
    ]);
    assert_eq!(database.compounds[0].orphan_ranges.len(), 1);
    assert!(database.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.kind,
        DiagnosticKind::OrphanHeatCapacityRange {
            phase_id_raw: 101,
            kind: HeatCapacityKind::Id3
        }
    )));
}

#[test]
fn orphan_kappa_is_preserved_and_reported() {
    let database = database([header(), compound("Orphan", "X"), kappa(101, 298.0, 1000.0)]);
    assert_eq!(database.compounds[0].orphan_ranges.len(), 1);
    assert!(database.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.kind,
        DiagnosticKind::OrphanKappaRange { phase_id_raw: 101 }
    )));
}

#[test]
fn invalid_chunk_ordering_is_fatal() {
    let bytes = [
        header(),
        compound("Bad", "X"),
        comment(b"too early"),
        ordinary(101, "late"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let error = Database::from_bytes(&bytes).expect_err("ordering must be fatal");
    assert!(matches!(
        error,
        DatabaseError::Domain(DomainError::Ordering {
            chunk_id: 7,
            compound_index: Some(0),
            ..
        })
    ));
}

#[test]
fn unknown_chunks_are_preserved_inside_a_group() {
    let database = database([
        header(),
        compound("Unknown", "X"),
        ordinary(101, "solid"),
        unknown(42),
    ]);
    assert_eq!(database.compounds[0].unknown_chunks.len(), 1);
    assert_eq!(database.compounds[0].unknown_chunks[0].id(), 42);
    assert!(
        database
            .diagnostics
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind, DiagnosticKind::UnknownChunk { id: 42 }))
    );
}

#[test]
fn decodes_all_phase_states_and_indexes() {
    let database = database([
        header(),
        compound("States", "X"),
        ordinary(101, "solid"),
        ordinary(802, "liquid"),
        ordinary(902, "gas"),
        ordinary(991, "aqueous"),
    ]);
    let phases = &database.compounds[0].phases;
    assert_eq!(phases[0].state(), PhaseState::Solid);
    assert_eq!(phases[0].index(), 1);
    assert_eq!(phases[1].state(), PhaseState::Liquid);
    assert_eq!(phases[1].index(), 2);
    assert_eq!(phases[2].state(), PhaseState::Gas);
    assert_eq!(phases[2].index(), 2);
    assert_eq!(phases[3].state(), PhaseState::Aqueous);
    assert_eq!(phases[3].index(), 1);
}

#[test]
fn generates_compact_and_chemapp_labels() {
    let database = database([
        header(),
        compound("Labels", "X"),
        ordinary(101, "solid"),
        ordinary(802, "liquid"),
        ordinary(902, "gas"),
        ordinary(991, "aqueous"),
    ]);
    let phases = &database.compounds[0].phases;
    assert_eq!(phases[0].compact_label(), "S1");
    assert_eq!(phases[0].chemapp_label(), "s");
    assert_eq!(phases[1].compact_label(), "L2");
    assert_eq!(phases[1].chemapp_label(), "l2");
    assert_eq!(phases[2].compact_label(), "G2");
    assert_eq!(phases[2].chemapp_label(), "g2");
    assert_eq!(phases[3].compact_label(), "AQ1");
    assert_eq!(phases[3].chemapp_label(), "aq");
}

#[test]
fn finds_compound_by_exact_formula() {
    let database = database([header(), compound("Water", "H2O"), compound("Salt", "NaCl")]);
    assert_eq!(
        database
            .find_compound_by_formula("NaCl")
            .unwrap()
            .name()
            .unwrap(),
        "Salt"
    );
    assert!(database.find_compound_by_formula("Na").is_none());
}

#[test]
fn finds_compound_by_name_prefix() {
    let database = database([
        header(),
        compound("Water vapor", "H2O"),
        compound("Salt", "NaCl"),
    ]);
    assert_eq!(
        database
            .find_compound_by_name("Water")
            .unwrap()
            .formula()
            .unwrap(),
        "H2O"
    );
}

#[test]
fn finds_phase_by_name_and_chemapp_label() {
    let database = database([
        header(),
        compound("Water", "H2O"),
        ordinary(101, "solid"),
        ordinary(802, "liquid"),
    ]);
    let compound = &database.compounds[0];
    assert_eq!(
        compound
            .find_phase_by_name("liquid")
            .unwrap()
            .phase_id_raw(),
        802
    );
    assert_eq!(
        compound
            .find_phase_by_chemapp_label("s")
            .unwrap()
            .name()
            .unwrap(),
        "solid"
    );
    assert!(compound.find_phase_by_name("LIQUID").is_none());
}

#[test]
fn raw_chunks_remain_accessible() {
    let database = database([
        header(),
        compound("Water", "H2O"),
        ordinary(101, "solid"),
        heat_capacity(6, 101, 298.0, 1000.0),
    ]);
    assert_eq!(database.compounds[0].raw.compound_name[0], b'W');
    assert_eq!(database.compounds[0].phases[0].raw.id(), 7);
    assert_eq!(
        database.compounds[0].phases[0].heat_capacity_ranges[0]
            .raw
            .phase_id_raw,
        101
    );
}

#[test]
fn reports_text_and_temperature_diagnostics_without_dropping_records() {
    let mut invalid_compound = compound("Invalid", "X");
    invalid_compound[32] = 0xff;
    let database = database([
        header(),
        invalid_compound,
        ordinary(0, "questionable"),
        heat_capacity(2, 0, f64::NAN, 1000.0),
    ]);
    assert_eq!(database.compounds[0].phases.len(), 1);
    assert!(
        database
            .diagnostics
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind, DiagnosticKind::InvalidAsciiText { .. }))
    );
    assert!(database.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.kind,
        DiagnosticKind::SuspiciousPhaseId {
            phase_id_raw: 0,
            index: -100,
            ..
        }
    )));
    assert!(database.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.kind,
        DiagnosticKind::NonFiniteTemperatureRange {
            phase_id_raw: 0,
            ..
        }
    )));
}
