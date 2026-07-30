use factsage_compound_parser::{
    DatabaseEditor, EditError, EnergyUnit, PhaseKind, RawChunk, UnitError,
};

const CHUNK_SIZE: usize = 256;

fn put_f64(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: f64) {
    chunk[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn put_i32(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: i32) {
    chunk[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: u32) {
    chunk[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn header() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0xaa; CHUNK_SIZE];
    chunk[0] = 9;
    chunk[2..6].copy_from_slice(b"CMPD");
    chunk
}

fn compound(energy_unit: u32) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0xbb; CHUNK_SIZE];
    chunk[0] = 1;
    chunk[32..72].fill(0x20);
    chunk[32..36].copy_from_slice(b"old ");
    put_u32(&mut chunk, 156, energy_unit);
    for index in 0..7 {
        put_f64(&mut chunk, 176 + index * 8, index as f64 + 1.0);
    }
    chunk
}

fn ordinary() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0xcc; CHUNK_SIZE];
    chunk[0] = 7;
    put_f64(&mut chunk, 32, 10.0);
    put_f64(&mut chunk, 40, 2.0);
    put_i32(&mut chunk, 48, -101);
    put_i32(&mut chunk, 52, 101);
    chunk[136..176].fill(0x20);
    chunk[136..140].copy_from_slice(b"old ");
    chunk
}

fn transition() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0xdd; CHUNK_SIZE];
    chunk[0] = 8;
    put_f64(&mut chunk, 32, 20.0);
    put_f64(&mut chunk, 40, 500.0);
    put_i32(&mut chunk, 48, 101);
    put_i32(&mut chunk, 52, 201);
    chunk[136..176].fill(0x20);
    chunk[136..140].copy_from_slice(b"old ");
    chunk
}

fn heat_capacity() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0xee; CHUNK_SIZE];
    chunk[0] = 2;
    put_f64(&mut chunk, 32, 123.0);
    put_f64(&mut chunk, 40, 456.0);
    put_i32(&mut chunk, 48, 101);
    put_f64(&mut chunk, 56, 100.0);
    put_f64(&mut chunk, 64, 200.0);
    put_f64(&mut chunk, 72, 1.0);
    chunk
}

fn unknown() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0x5a; CHUNK_SIZE];
    chunk[0] = 250;
    chunk
}

fn input(energy_unit: u32) -> Vec<u8> {
    [
        header(),
        compound(energy_unit),
        ordinary(),
        transition(),
        heat_capacity(),
        unknown(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn assert_only_changed_within(
    before: &[u8],
    after: &[u8],
    allowed: impl IntoIterator<Item = std::ops::Range<usize>>,
) {
    let allowed = allowed.into_iter().collect::<Vec<_>>();
    assert_eq!(before.len(), after.len());
    for (index, (before_byte, after_byte)) in before.iter().zip(after).enumerate() {
        if before_byte != after_byte {
            assert!(
                allowed.iter().any(|range| range.contains(&index)),
                "unexpected changed byte at {index}"
            );
        }
    }
}

#[test]
fn edits_fixed_width_ascii_names_without_touching_other_bytes() {
    let before = input(1);
    let mut editor = DatabaseEditor::from_bytes(&before).unwrap();
    editor.set_compound_name(0, "new compound").unwrap();
    editor.set_phase_name(0, 0, "new phase").unwrap();
    let after = editor.to_bytes().unwrap();

    assert_eq!(&after[CHUNK_SIZE + 32..CHUNK_SIZE + 44], b"new compound");
    assert!(
        after[CHUNK_SIZE + 44..CHUNK_SIZE + 72]
            .iter()
            .all(|byte| *byte == 0)
    );
    assert_eq!(
        &after[2 * CHUNK_SIZE + 136..2 * CHUNK_SIZE + 145],
        b"new phase"
    );
    assert!(
        after[2 * CHUNK_SIZE + 145..2 * CHUNK_SIZE + 176]
            .iter()
            .all(|byte| *byte == 0)
    );
    assert_only_changed_within(
        &before,
        &after,
        [
            CHUNK_SIZE + 32..CHUNK_SIZE + 72,
            2 * CHUNK_SIZE + 136..2 * CHUNK_SIZE + 176,
        ],
    );
}

#[test]
fn validates_fixed_width_ascii_names() {
    let mut editor = DatabaseEditor::from_bytes(&input(1)).unwrap();
    assert!(editor.set_compound_name(0, &"x".repeat(40)).is_ok());
    assert_eq!(
        editor.set_compound_name(0, &"x".repeat(41)),
        Err(EditError::TextTooLong {
            field: "compound_name",
            maximum: 40,
            actual: 41,
        })
    );
    assert_eq!(
        editor.set_phase_name(0, 0, "phase°"),
        Err(EditError::NonAsciiText {
            field: "phase_name",
            byte_index: 5,
        })
    );
    assert_eq!(
        editor.set_phase_name(0, 0, "a\0b"),
        Err(EditError::EmbeddedNul {
            field: "phase_name"
        })
    );
}

#[test]
fn applies_inverse_energy_conversion_to_ordinary_and_transition_fields() {
    let before = input(0);
    let mut editor = DatabaseEditor::from_bytes(&before).unwrap();
    editor
        .set_ordinary_phase_enthalpy_298_j_per_mol(0, 0, 41.84)
        .unwrap();
    editor
        .set_ordinary_phase_entropy_298_j_per_mol_k(0, 0, 8.368)
        .unwrap();
    editor
        .set_transition_phase_enthalpy_j_per_mol(0, 1, 83.68)
        .unwrap();
    editor
        .set_transition_phase_temperature_k(0, 1, 600.0)
        .unwrap();

    let raw = editor.raw();
    let RawChunk::PhaseOrdinary(ordinary) = &raw.chunks[2] else {
        panic!("fixture contains ordinary phase");
    };
    assert_eq!(ordinary.enthalpy, 10.0);
    assert_eq!(ordinary.entropy, 2.0);
    let RawChunk::PhaseTransition(transition) = &raw.chunks[3] else {
        panic!("fixture contains transition phase");
    };
    assert_eq!(transition.transition_enthalpy, 20.0);
    assert_eq!(transition.transition_temperature, 600.0);
    let RawChunk::HeatCapacity { chunk: cp, .. } = &raw.chunks[4] else {
        panic!("fixture contains CP range");
    };
    assert_eq!(cp.enthalpy, 123.0);
    assert_eq!(cp.entropy, 456.0);

    let domain = editor.domain().unwrap();
    assert_eq!(
        domain.compounds[0].enthalpy_298_j_per_mol(0).unwrap(),
        41.84
    );
    assert_eq!(
        domain.compounds[0]
            .transition_enthalpy_j_per_mol(1)
            .unwrap(),
        83.68
    );
    assert_eq!(
        domain.compounds[0].phases[1]
            .transition_temperature_k()
            .unwrap(),
        600.0
    );
}

#[test]
fn retains_joule_values_and_rejects_unknown_units_and_wrong_phase_setters() {
    let mut joules = DatabaseEditor::from_bytes(&input(1)).unwrap();
    joules
        .set_transition_phase_enthalpy_j_per_mol(0, 1, 17.0)
        .unwrap();
    let RawChunk::PhaseTransition(transition) = &joules.raw().chunks[3] else {
        panic!("fixture contains transition phase");
    };
    assert_eq!(transition.transition_enthalpy, 17.0);

    assert_eq!(
        joules.set_transition_phase_enthalpy_j_per_mol(0, 0, 17.0),
        Err(EditError::WrongPhaseType {
            compound_index: 0,
            phase_index: 0,
            expected: PhaseKind::Transition,
            actual: PhaseKind::Ordinary,
        })
    );

    let mut unknown = DatabaseEditor::from_bytes(&input(99)).unwrap();
    assert_eq!(
        unknown.set_ordinary_phase_enthalpy_298_j_per_mol(0, 0, 1.0),
        Err(EditError::Unit(UnitError::UnknownEnergyUnit { raw: 99 }))
    );
}

#[test]
fn edits_stoichiometry_and_rebuilds_domain_links_from_raw() {
    let before = input(1);
    let mut editor = DatabaseEditor::from_bytes(&before).unwrap();
    let coefficients = [0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5];
    editor
        .set_real_stoichiometric_coefficients(0, coefficients)
        .unwrap();
    assert!(matches!(
        editor.set_real_stoichiometric_coefficients(0, [f64::NAN; 7]),
        Err(EditError::NonFiniteValue {
            field: "real_stoichiometric_coefficients",
            value,
        }) if value.is_nan()
    ));

    let domain = editor.domain().unwrap();
    assert_eq!(
        domain.compounds[0].real_stoichiometric_coefficients(),
        &coefficients
    );
    assert_eq!(domain.compounds[0].phases.len(), 2);
    assert_eq!(domain.compounds[0].phases[0].heat_capacity_ranges.len(), 1);
    assert_eq!(domain.compounds[0].unknown_chunks.len(), 1);
    assert!(matches!(
        domain.compounds[0].unknown_chunks[0],
        RawChunk::Unknown { id: 250, .. }
    ));

    let after = editor.to_bytes().unwrap();
    assert_only_changed_within(
        &before,
        &after,
        std::iter::once(CHUNK_SIZE + 176..CHUNK_SIZE + 232),
    );
}

#[test]
fn reports_missing_records_and_exposes_the_authoritative_raw_unit() {
    let mut editor = DatabaseEditor::from_bytes(&input(1)).unwrap();
    assert_eq!(editor.raw().chunks.len(), 6);
    assert_eq!(
        editor.set_compound_name(1, "missing"),
        Err(EditError::CompoundNotFound { compound_index: 1 })
    );
    assert_eq!(
        editor.set_phase_name(0, 2, "missing"),
        Err(EditError::PhaseNotFound {
            compound_index: 0,
            phase_index: 2,
        })
    );
    assert_eq!(
        EnergyUnit::from_raw(match &editor.raw().chunks[1] {
            RawChunk::Compound(chunk) => chunk.unit_energy,
            _ => unreachable!(),
        }),
        EnergyUnit::Joules
    );
}
