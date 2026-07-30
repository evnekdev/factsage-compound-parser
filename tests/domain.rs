use factsage_compound_parser::{
    Database, DatabaseError, DatabaseView, DiagnosticKind, DomainError, DomainIndex,
    HeatCapacityKind, PhaseState, RawDatabase,
};

const CHUNK_SIZE: usize = 256;

fn put<const N: usize>(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: [u8; N]) {
    chunk[offset..offset + N].copy_from_slice(&value);
}

fn put_ascii(chunk: &mut [u8; CHUNK_SIZE], offset: usize, text: &[u8]) {
    chunk[offset..offset + text.len()].copy_from_slice(text);
}

fn put_i32(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: i32) {
    put(chunk, offset, value.to_le_bytes());
}

fn put_f64(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: f64) {
    put(chunk, offset, value.to_le_bytes());
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
    put_i32(&mut chunk, 48, phase_id_raw);
    put_f64(&mut chunk, 56, t_min);
    put_f64(&mut chunk, 64, t_max);
    chunk
}

fn kappa(phase_id_raw: i32) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 11;
    put_i32(&mut chunk, 48, phase_id_raw);
    chunk
}

fn comment(bytes: &[u8]) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 10;
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
fn groups_phases_ranges_and_comments_as_borrowed_views() {
    let database = database([
        header(),
        compound("Water", "H2O"),
        ordinary(101, "solid"),
        ordinary(802, "liquid"),
        transition(902, 802, "gas transition"),
        heat_capacity(2, 101, 298.0, 500.0),
        heat_capacity(5, 101, 500.0, 1000.0),
        kappa(802),
        comment(b"hello"),
        comment(&[0xb0, b' ', b'w', b'o', b'r', b'l', b'd']),
    ]);
    let view = database.view().unwrap();
    assert_eq!(view.compound_count(), 1);
    assert!(view.diagnostics().is_empty());

    let compound = view.compounds().next().unwrap();
    assert_eq!(compound.name().unwrap(), "Water");
    assert_eq!(compound.formula().unwrap(), "H2O");
    assert_eq!(compound.phase_count(), 3);
    assert_eq!(compound.joined_comment(), "hello° world");
    assert_eq!(compound.raw().compound_name[0], b'W');

    let phases = compound.phases().collect::<Vec<_>>();
    assert_eq!(phases[0].phase_id_raw(), 101);
    assert!(!phases[0].is_transition());
    assert!(phases[2].is_transition());
    assert_eq!(phases[0].heat_capacity_range_count(), 2);
    assert_eq!(phases[1].physical_property_range_count(), 1);
    assert_eq!(
        phases[0]
            .heat_capacity_ranges()
            .map(|range| range.kind())
            .collect::<Vec<_>>(),
        vec![HeatCapacityKind::Id2, HeatCapacityKind::Id5]
    );
    assert_eq!(
        phases[0]
            .heat_capacity_ranges()
            .next()
            .unwrap()
            .raw()
            .phase_id_raw,
        101
    );
}

#[test]
fn reports_duplicates_orphans_and_unknowns_without_copying_them() {
    let database = database([
        header(),
        compound("Duplicate", "X"),
        ordinary(101, "one"),
        ordinary(101, "two"),
        heat_capacity(3, 101, 298.0, 1000.0),
        unknown(42),
        compound("Orphan", "Y"),
        heat_capacity(6, 201, 298.0, 1000.0),
        kappa(202),
    ]);
    let view = database.view().unwrap();
    let compounds = view.compounds().collect::<Vec<_>>();
    assert_eq!(compounds[0].phase_count(), 2);
    assert_eq!(compounds[0].orphan_range_count(), 1);
    assert_eq!(compounds[0].unknown_chunks().next().unwrap().id(), 42);
    assert_eq!(compounds[1].orphan_range_count(), 2);
    assert!(view.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic.kind,
        DiagnosticKind::DuplicatePhaseId { phase_id_raw: 101 }
    )));
    assert!(view.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic.kind,
        DiagnosticKind::AmbiguousPhaseLink {
            phase_id_raw: 101,
            ..
        }
    )));
    assert!(view.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic.kind,
        DiagnosticKind::OrphanHeatCapacityRange {
            phase_id_raw: 201,
            ..
        }
    )));
    assert!(view.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic.kind,
        DiagnosticKind::OrphanKappaRange { phase_id_raw: 202 }
    )));
    assert!(
        view.diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind, DiagnosticKind::UnknownChunk { id: 42 }))
    );
}

#[test]
fn rejects_invalid_known_chunk_ordering() {
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
fn decodes_phase_states_labels_and_lookups() {
    let database = database([
        header(),
        compound("States", "X"),
        ordinary(101, "solid"),
        ordinary(802, "liquid"),
        ordinary(902, "gas"),
        ordinary(991, "aqueous"),
    ]);
    let view = database.view().unwrap();
    let compound = view.find_compound_by_name("Sta").unwrap();
    let phases = compound.phases().collect::<Vec<_>>();
    assert_eq!(phases[0].state(), PhaseState::Solid);
    assert_eq!(phases[0].index(), 1);
    assert_eq!(phases[1].state(), PhaseState::Liquid);
    assert_eq!(phases[1].compact_label(), "L2");
    assert_eq!(phases[2].state(), PhaseState::Gas);
    assert_eq!(phases[2].chemapp_label(), "g2");
    assert_eq!(phases[3].state(), PhaseState::Aqueous);
    assert_eq!(phases[3].chemapp_label(), "aq");
    assert_eq!(
        compound.find_phase_by_raw_id(802).unwrap().name().unwrap(),
        "liquid"
    );
    assert_eq!(
        compound.find_phase_by_name("gas").unwrap().phase_id_raw(),
        902
    );
    assert_eq!(
        compound
            .find_phase_by_chemapp_label("s")
            .unwrap()
            .phase_id_raw(),
        101
    );
    assert_eq!(view.find_compound_by_formula("X").unwrap().chunk_index(), 1);
}

#[test]
fn detects_stale_or_foreign_indexes_before_creating_views() {
    let bytes = [header(), compound("One", "X")]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let mut raw = RawDatabase::from_bytes(&bytes).unwrap();
    let index = DomainIndex::build(&raw).unwrap();
    let _ = raw.chunks_mut();
    assert!(matches!(
        DatabaseView::new(&raw, &index),
        Err(DomainError::StaleIndex { .. })
    ));
}
