use std::io::Cursor;

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use factsage_compound_parser::{Database, DatabaseEditor, RawChunk, RawDatabase};

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

fn cp() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0_u8; CHUNK_SIZE];
    chunk[0] = 2;
    chunk[52..56].copy_from_slice(&101_i32.to_le_bytes());
    chunk[56..64].copy_from_slice(&300.0_f64.to_le_bytes());
    chunk[64..72].copy_from_slice(&2_000.0_f64.to_le_bytes());
    chunk
}

fn synthetic_bytes(target_bytes: usize) -> Vec<u8> {
    let target_chunks = target_bytes.div_ceil(CHUNK_SIZE).max(4);
    let mut chunks = vec![header()];
    while chunks.len() + 3 <= target_chunks {
        chunks.push(compound());
        chunks.push(ordinary());
        chunks.push(cp());
    }
    chunks.into_iter().flatten().collect()
}

fn benchmarks(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("architecture");
    for size in [256 * 1024, 1024 * 1024, 5 * 1024 * 1024, 7 * 1024 * 1024] {
        let bytes = synthetic_bytes(size);
        let raw = RawDatabase::from_bytes(&bytes).expect("synthetic input is valid");
        let actual_size = bytes.len();

        group.bench_with_input(
            BenchmarkId::new("from_bytes", actual_size),
            &bytes,
            |bench, bytes| {
                bench.iter(|| {
                    RawDatabase::from_bytes(black_box(bytes)).expect("synthetic input is valid")
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("from_reader", actual_size),
            &bytes,
            |bench, bytes| {
                bench.iter(|| {
                    RawDatabase::from_reader(Cursor::new(black_box(bytes)))
                        .expect("synthetic input is valid")
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("domain_index", actual_size),
            &raw,
            |bench, raw| {
                bench.iter(|| {
                    Database::from_raw(black_box(raw.clone())).expect("synthetic input is valid")
                });
            },
        );
        let database = Database::from_raw(raw.clone()).expect("synthetic input is valid");
        group.bench_with_input(
            BenchmarkId::new("view_traversal", actual_size),
            &database,
            |bench, database| {
                bench.iter(|| {
                    let view = database
                        .view()
                        .expect("owned database index remains current");
                    black_box(
                        view.compounds()
                            .map(|compound| compound.phases().count())
                            .sum::<usize>(),
                    );
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("serialize_vec", actual_size),
            &raw,
            |bench, raw| {
                bench.iter(|| raw.to_bytes().expect("synthetic stream serializes"));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("serialize_stream", actual_size),
            &raw,
            |bench, raw| {
                bench.iter(|| {
                    let mut sink = Vec::with_capacity(raw.chunks().len() * CHUNK_SIZE);
                    raw.write_to(&mut sink)
                        .expect("synthetic stream serializes");
                    black_box(sink);
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert_middle", actual_size),
            &raw,
            |bench, raw| {
                bench.iter(|| {
                    let mut database = raw.clone();
                    database
                        .insert_chunk(
                            database.chunks().len() / 2,
                            RawChunk::Unknown {
                                id: 250,
                                body: [0; 255],
                            },
                        )
                        .expect("middle index is valid");
                    black_box(database);
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("editor_rebuild", actual_size),
            &raw,
            |bench, raw| {
                bench.iter(|| {
                    let mut editor = DatabaseEditor::from_raw(raw.clone());
                    editor.rebuild_index().expect("synthetic input is valid");
                    black_box(editor);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(architecture, benchmarks);
criterion_main!(architecture);
