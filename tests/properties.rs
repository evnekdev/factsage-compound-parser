use factsage_compound_parser::{Database, DatabaseEditor, RawChunk, RawDatabase};
use proptest::prelude::*;

const CHUNK_SIZE: usize = 256;

fn header() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 9;
    chunk[2..6].copy_from_slice(b"CMPD");
    chunk
}

fn compound() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 1;
    chunk
}

fn ordinary() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 7;
    chunk[52..56].copy_from_slice(&101_i32.to_le_bytes());
    chunk
}

fn transition() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 8;
    chunk[52..56].copy_from_slice(&201_i32.to_le_bytes());
    chunk
}

fn cp() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 2;
    chunk[48..52].copy_from_slice(&101_i32.to_le_bytes());
    chunk[56..64].copy_from_slice(&300.0_f64.to_le_bytes());
    chunk[64..72].copy_from_slice(&1_000.0_f64.to_le_bytes());
    chunk
}

fn bytes(chunks: impl IntoIterator<Item = [u8; CHUNK_SIZE]>) -> Vec<u8> {
    chunks.into_iter().flatten().collect()
}

fn raw_with_unknown(id: u8, body: &[u8]) -> Vec<u8> {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = id;
    chunk[1..].copy_from_slice(body);
    bytes([header(), chunk])
}

fn assert_only_changed_in(
    before: &[u8],
    after: &[u8],
    allowed: std::ops::Range<usize>,
) -> Result<(), TestCaseError> {
    prop_assert_eq!(before.len(), after.len());
    for (index, (left, right)) in before.iter().zip(after).enumerate() {
        if left != right {
            prop_assert!(
                allowed.contains(&index),
                "changed byte {index} outside {allowed:?}"
            );
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn valid_raw_streams_round_trip_exactly(
        body in prop::collection::vec(any::<u8>(), 255),
        id in any::<u8>().prop_filter("unknown ID", |id| !(1..=11).contains(id)),
    ) {
        let input = raw_with_unknown(id, &body);
        let parsed = RawDatabase::from_bytes(&input)?;
        prop_assert_eq!(parsed.to_bytes()?, input);
    }

    #[test]
    fn arbitrary_float_bit_patterns_survive_known_chunk_round_trip(
        enthalpy_bits in any::<u64>(),
        entropy_bits in any::<u64>(),
        density_bits in any::<u64>(),
        expansion_bits in any::<u32>(),
    ) {
        let mut phase = ordinary();
        phase[33..41].copy_from_slice(&enthalpy_bits.to_le_bytes());
        phase[41..49].copy_from_slice(&entropy_bits.to_le_bytes());
        phase[57..65].copy_from_slice(&density_bits.to_le_bytes());
        phase[65..69].copy_from_slice(&expansion_bits.to_le_bytes());
        let input = bytes([header(), compound(), phase]);
        let parsed = RawDatabase::from_bytes(&input)?;
        let output = parsed.to_bytes()?;
        prop_assert_eq!(&output[CHUNK_SIZE + 32..CHUNK_SIZE + 40], &compound()[32..40]);
        prop_assert_eq!(output, input);
    }

    #[test]
    fn unknown_chunk_bodies_are_preserved(
        id in any::<u8>().prop_filter("unknown ID", |id| !(1..=11).contains(id)),
        body in prop::collection::vec(any::<u8>(), 255),
    ) {
        let input = raw_with_unknown(id, &body);
        let parsed = RawDatabase::from_bytes(&input)?;
        let Some(RawChunk::Unknown { id: parsed_id, body: parsed_body }) = parsed.chunks().get(1) else {
            prop_assert!(false, "unknown chunk changed variant");
            return Ok(());
        };
        prop_assert_eq!(*parsed_id, id);
        prop_assert_eq!(parsed_body.as_slice(), body.as_slice());
    }

    #[test]
    fn insertion_and_removal_recover_original_raw_bytes(
        insertion_after_header in 1_usize..4,
        id in any::<u8>().prop_filter("unknown ID", |id| !(1..=11).contains(id)),
        body in prop::collection::vec(any::<u8>(), 255),
    ) {
        let original = bytes([header(), compound(), ordinary()]);
        let mut raw = RawDatabase::from_bytes(&original)?;
        let index = insertion_after_header.min(raw.chunks().len());
        let mut body_array = [0_u8; 255];
        body_array.copy_from_slice(&body);
        let inserted = RawChunk::Unknown { id, body: body_array };
        raw.insert_chunk(index, inserted.clone())?;
        let reparsed = RawDatabase::from_bytes(&raw.to_bytes()?)?;
        prop_assert_eq!(reparsed.chunks().get(index), Some(&inserted));
        let removed = raw.remove_chunk(index)?;
        prop_assert_eq!(removed, inserted);
        prop_assert_eq!(raw.to_bytes()?, original);
    }

    #[test]
    fn non_structural_name_edits_are_byte_local(
        name in "[A-Za-z0-9 ]{0,40}",
    ) {
        let original = bytes([header(), compound(), ordinary(), transition()]);
        let mut editor = DatabaseEditor::from_bytes(&original)?;
        editor.set_compound_name(0, &name)?;
        let output = editor.to_bytes()?;
        assert_only_changed_in(&original, &output, CHUNK_SIZE + 32..CHUNK_SIZE + 72)?;
    }

    #[test]
    fn structural_edits_rebuild_views_and_resolve_indexes(
        body in prop::collection::vec(any::<u8>(), 255),
    ) {
        let input = bytes([header(), compound(), ordinary(), cp()]);
        let mut editor = DatabaseEditor::from_bytes(&input)?;
        let _ = editor.view()?;
        prop_assert!(editor.has_current_index());
        let mut body_array = [0_u8; 255];
        body_array.copy_from_slice(&body);
        editor.insert_chunk(4, RawChunk::Unknown { id: 250, body: body_array })?;
        prop_assert!(!editor.has_current_index());
        let view = editor.view()?;
        let Some(compound) = view.compounds().next() else { prop_assert!(false, "missing compound"); return Ok(()); };
        prop_assert_eq!(compound.phase_count(), 1);
        prop_assert_eq!(compound.unknown_chunks().count(), 1);
        let Some(phase) = compound.phases().next() else { prop_assert!(false, "missing phase"); return Ok(()); };
        prop_assert_eq!(phase.chunk_index(), 2);
        prop_assert_eq!(phase.heat_capacity_ranges().count(), 1);
    }

    #[test]
    fn repeated_view_traversal_matches_direct_raw_order(
        use_transition in any::<bool>(),
    ) {
        let phase = if use_transition { transition() } else { ordinary() };
        let input = bytes([header(), compound(), phase]);
        let database = Database::from_bytes(&input)?;
        let view = database.view()?;
        let first = view.compounds().flat_map(|compound| compound.phases()).map(|phase| phase.chunk_index()).collect::<Vec<_>>();
        let second = view.compounds().flat_map(|compound| compound.phases()).map(|phase| phase.chunk_index()).collect::<Vec<_>>();
        prop_assert_eq!(&first, &second);
        prop_assert_eq!(first, vec![2]);
    }
}
