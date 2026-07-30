use factsage_compound_parser::{DatabaseEditor, EditError, EnergyUnit, RawChunk, RawEditError};

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
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 9;
    chunk[2..6].copy_from_slice(b"CMPD");
    chunk
}

fn compound(energy_unit: u32) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0xaa; CHUNK_SIZE];
    chunk[0] = 1;
    put_u32(&mut chunk, 156, energy_unit);
    for index in 0..7 {
        put_f64(&mut chunk, 176 + index * 8, index as f64 + 1.0);
    }
    chunk
}

fn ordinary() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0xbb; CHUNK_SIZE];
    chunk[0] = 7;
    put_f64(&mut chunk, 32, 10.0);
    put_f64(&mut chunk, 40, 2.0);
    put_i32(&mut chunk, 48, -101);
    put_i32(&mut chunk, 52, 101);
    chunk
}

fn transition() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0xcc; CHUNK_SIZE];
    chunk[0] = 8;
    put_f64(&mut chunk, 32, 20.0);
    put_f64(&mut chunk, 40, 500.0);
    put_i32(&mut chunk, 48, 101);
    put_i32(&mut chunk, 52, 201);
    chunk
}

fn input(energy_unit: u32) -> Vec<u8> {
    [header(), compound(energy_unit), ordinary(), transition()]
        .into_iter()
        .flatten()
        .collect()
}

#[test]
fn non_structural_edits_keep_cached_index_and_change_only_documented_fields() {
    let before = input(0);
    let mut editor = DatabaseEditor::from_bytes(&before).unwrap();
    assert!(!editor.has_current_index());
    assert_eq!(editor.view().unwrap().compound_count(), 1);
    assert!(editor.has_current_index());

    editor.set_compound_name(0, "new compound").unwrap();
    editor.set_phase_name(0, 0, "new phase").unwrap();
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
    assert!(editor.has_current_index());

    let bytes = editor.to_bytes().unwrap();
    assert_eq!(&bytes[CHUNK_SIZE + 32..CHUNK_SIZE + 44], b"new compound");
    assert_eq!(
        &bytes[2 * CHUNK_SIZE + 136..2 * CHUNK_SIZE + 145],
        b"new phase"
    );
    let RawChunk::PhaseOrdinary(ordinary) = &editor.raw().chunks()[2] else {
        panic!("synthetic stream contains ordinary phase");
    };
    assert_eq!(ordinary.enthalpy, 10.0);
    assert_eq!(ordinary.entropy, 2.0);
    let RawChunk::PhaseTransition(transition) = &editor.raw().chunks()[3] else {
        panic!("synthetic stream contains transition phase");
    };
    assert_eq!(transition.transition_enthalpy, 20.0);
    assert_eq!(transition.transition_temperature, 600.0);
    let compound = editor.view().unwrap().compounds().next().unwrap();
    assert_eq!(compound.enthalpy_298_j_per_mol(0).unwrap(), 41.84);
    assert_eq!(compound.transition_enthalpy_j_per_mol(1).unwrap(), 83.68);
}

#[test]
fn structural_edits_invalidate_and_rebuild_index_without_losing_raw_order() {
    let original = input(1);
    let mut editor = DatabaseEditor::from_bytes(&original).unwrap();
    let _ = editor.view().unwrap();
    assert!(editor.has_current_index());
    let inserted = RawChunk::Unknown {
        id: 250,
        body: [7; 255],
    };
    editor.insert_chunk(4, inserted.clone()).unwrap();
    assert!(!editor.has_current_index());
    let view = editor.view().unwrap();
    assert_eq!(view.compounds().next().unwrap().unknown_chunks().count(), 1);
    let removed = editor.remove_chunk(4).unwrap();
    assert_eq!(removed, inserted);
    assert!(!editor.has_current_index());
    assert_eq!(editor.to_bytes().unwrap(), original);
    assert_eq!(
        editor.insert_chunk(
            99,
            RawChunk::Unknown {
                id: 1,
                body: [0; 255]
            }
        ),
        Err(RawEditError::IndexOutOfBounds {
            operation: "insert",
            index: 99,
            len: 4,
        })
    );
}

#[test]
fn validates_edit_inputs_and_exposes_raw_stream() {
    let mut editor = DatabaseEditor::from_bytes(&input(1)).unwrap();
    assert_eq!(editor.raw().chunks().len(), 4);
    assert_eq!(
        editor.set_compound_name(0, &"x".repeat(41)),
        Err(EditError::TextTooLong {
            field: "compound_name",
            maximum: 40,
            actual: 41,
        })
    );
    assert!(matches!(
        editor.set_phase_name(0, 0, "phase°"),
        Err(EditError::NonAsciiText { .. })
    ));
    assert_eq!(
        EnergyUnit::from_raw(match &editor.raw().chunks()[1] {
            RawChunk::Compound(chunk) => chunk.unit_energy,
            _ => 99,
        }),
        EnergyUnit::Joules
    );
}
