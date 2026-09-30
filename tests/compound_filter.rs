use factsage_compound_parser::filter::{CompoundFilterError, CompoundFilterPlan};
use factsage_compound_parser::{DomainIndex, RawChunk, RawDatabase};

fn header() -> [u8; 256] {
    let mut value = [0; 256];
    value[0] = 9;
    value[2..6].copy_from_slice(b"CMPD");
    value
}

fn compound() -> [u8; 256] {
    let mut value = [0; 256];
    value[0] = 1;
    value
}

fn phase(id: i32) -> [u8; 256] {
    let mut value = [0; 256];
    value[0] = 7;
    value[48..52].copy_from_slice(&(-id).to_le_bytes());
    value[52..56].copy_from_slice(&id.to_le_bytes());
    value
}

fn range(kind: u8, id: i32) -> [u8; 256] {
    let mut value = [0; 256];
    value[0] = kind;
    value[48..52].copy_from_slice(&id.to_le_bytes());
    value[56..64].copy_from_slice(&298.15_f64.to_le_bytes());
    value[64..72].copy_from_slice(&1000.0_f64.to_le_bytes());
    value
}

#[test]
fn selected_group_retains_all_native_dependencies_and_opaque_records() {
    let mut unknown = [0; 256];
    unknown[0] = 77;
    unknown[100] = 42;
    let mut comment = [0; 256];
    comment[0] = 10;
    let source = RawDatabase::from_bytes(
        &[
            header(),
            compound(),
            phase(101),
            range(2, 101),
            compound(),
            phase(102),
            range(2, 102),
            range(11, 102),
            comment,
            unknown,
        ]
        .concat(),
    )
    .unwrap();
    let plan = CompoundFilterPlan::new(&source, [1]).unwrap();
    let output = plan.materialize(&source).unwrap();
    assert_eq!(
        output.chunks().iter().map(RawChunk::id).collect::<Vec<_>>(),
        [9, 1, 7, 2, 11, 10, 77]
    );
    assert_eq!(output.chunks()[0], source.chunks()[0]);
    assert_eq!(output.chunks()[1..], source.chunks()[4..]);
    assert!(
        matches!(output.chunks().last(), Some(RawChunk::Unknown { id: 77, body }) if body[99] == 42)
    );
    assert_eq!(DomainIndex::build(&output).unwrap().compound_count(), 1);
}

#[test]
fn rejects_selected_orphan_and_stale_source() {
    let source =
        RawDatabase::from_bytes(&[header(), compound(), phase(101), range(2, 102)].concat())
            .unwrap();
    assert!(matches!(
        CompoundFilterPlan::new(&source, [0]),
        Err(CompoundFilterError::AmbiguousDependency { .. })
    ));
    let source =
        RawDatabase::from_bytes(&[header(), compound(), phase(101), range(2, 101)].concat())
            .unwrap();
    let plan = CompoundFilterPlan::new(&source, [0]).unwrap();
    assert!(matches!(
        CompoundFilterPlan::new(&source, [0, 0]),
        Err(CompoundFilterError::DuplicateCompoundIndex { .. })
    ));
    let mut changed = source.clone();
    changed.chunks_mut();
    assert!(matches!(
        plan.materialize(&changed),
        Err(CompoundFilterError::StaleSource)
    ));
}

#[test]
fn retained_nan_payload_is_verified_by_bytes() {
    let mut cp = range(2, 101);
    cp[72..80].copy_from_slice(&f64::from_bits(0x7ff8_0000_0000_1234).to_le_bytes());
    let source = RawDatabase::from_bytes(&[header(), compound(), phase(101), cp].concat()).unwrap();
    let selected = CompoundFilterPlan::new(&source, [0])
        .unwrap()
        .materialize(&source)
        .unwrap();
    assert_eq!(selected.to_bytes().unwrap(), source.to_bytes().unwrap());
}

#[test]
fn empty_selection_retains_only_the_native_header() {
    let source = RawDatabase::from_bytes(&[header(), compound(), phase(101)].concat()).unwrap();
    let plan = CompoundFilterPlan::new(&source, []).unwrap();
    let output = plan.materialize(&source).unwrap();
    assert_eq!(output.to_bytes().unwrap(), header());
    assert_eq!(DomainIndex::build(&output).unwrap().compound_count(), 0);
    assert!(matches!(
        CompoundFilterPlan::new(&source, [1]),
        Err(CompoundFilterError::InvalidCompoundIndex { index: 1, count: 1 })
    ));
}
