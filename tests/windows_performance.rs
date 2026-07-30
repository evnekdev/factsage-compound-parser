use std::fs;
use std::hint::black_box;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use factsage_compound_parser::{DatabaseEditor, DatabaseView, DomainIndex, RawChunk, RawDatabase};

const ITERATIONS: u32 = 12;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("MS16BASE.CDB")
}

#[test]
#[ignore = "requires the ignored local MS16BASE.CDB fixture and is intended for Windows release measurements"]
fn windows_release_measurement_uses_private_fixture_in_memory_only() {
    let path = fixture_path();
    if !path.is_file() {
        eprintln!("skipping Windows fixture measurement: local fixture is unavailable");
        return;
    }
    let bytes = fs::read(&path).expect("fixture must be readable");
    let mut from_bytes = Duration::ZERO;
    let mut from_reader = Duration::ZERO;
    let mut from_path = Duration::ZERO;
    let mut index = Duration::ZERO;
    let mut traverse = Duration::ZERO;
    let mut to_bytes = Duration::ZERO;
    let mut write_to = Duration::ZERO;
    let mut insert_middle = Duration::ZERO;
    let mut rebuild = Duration::ZERO;
    let mut raw = None;

    for _ in 0..ITERATIONS {
        let started = Instant::now();
        let parsed = RawDatabase::from_bytes(&bytes).expect("fixture parses from bytes");
        from_bytes += started.elapsed();

        let started = Instant::now();
        let streamed = RawDatabase::from_reader(Cursor::new(&bytes)).expect("fixture streams");
        from_reader += started.elapsed();
        assert_eq!(streamed, parsed);

        let started = Instant::now();
        let from_disk = RawDatabase::from_path(&path).expect("fixture parses from path");
        from_path += started.elapsed();
        assert_eq!(from_disk, parsed);

        let started = Instant::now();
        let domain_index = DomainIndex::build(&parsed).expect("fixture indexes");
        index += started.elapsed();

        let started = Instant::now();
        let view = DatabaseView::new(&parsed, &domain_index).expect("fresh view is valid");
        let count = view
            .compounds()
            .map(|compound| compound.phases().count())
            .sum::<usize>();
        black_box(count);
        traverse += started.elapsed();

        let started = Instant::now();
        let serialized = parsed.to_bytes().expect("fixture serializes");
        to_bytes += started.elapsed();
        assert_eq!(serialized, bytes);

        let started = Instant::now();
        let mut sink = Vec::with_capacity(bytes.len());
        parsed
            .write_to(&mut sink)
            .expect("fixture streams to memory");
        write_to += started.elapsed();
        assert_eq!(sink, bytes);

        let started = Instant::now();
        let mut inserted = parsed.clone();
        inserted
            .insert_chunk(
                inserted.chunks().len() / 2,
                RawChunk::Unknown {
                    id: 250,
                    body: [0; 255],
                },
            )
            .expect("middle insertion is representable");
        insert_middle += started.elapsed();

        let started = Instant::now();
        let mut editor = DatabaseEditor::from_raw(parsed.clone());
        editor.rebuild_index().expect("fixture editor rebuilds");
        rebuild += started.elapsed();
        raw = Some(parsed);
    }

    let raw = raw.expect("measurement loop ran");
    println!(
        "fixture_bytes={} chunks={} iterations={} from_bytes_us={} from_reader_us={} from_path_us={} index_us={} traverse_us={} to_bytes_us={} write_to_us={} insert_middle_us={} editor_rebuild_us={}",
        bytes.len(),
        raw.chunks().len(),
        ITERATIONS,
        from_bytes.as_micros() / u128::from(ITERATIONS),
        from_reader.as_micros() / u128::from(ITERATIONS),
        from_path.as_micros() / u128::from(ITERATIONS),
        index.as_micros() / u128::from(ITERATIONS),
        traverse.as_micros() / u128::from(ITERATIONS),
        to_bytes.as_micros() / u128::from(ITERATIONS),
        write_to.as_micros() / u128::from(ITERATIONS),
        insert_middle.as_micros() / u128::from(ITERATIONS),
        rebuild.as_micros() / u128::from(ITERATIONS),
    );
}
