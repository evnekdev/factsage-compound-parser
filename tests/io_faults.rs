use std::error::Error as _;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use factsage_compound_parser::{ParseError, RawDatabase, SerializeError};

const CHUNK_SIZE: usize = 256;

struct FailingReader {
    bytes: Vec<u8>,
    position: usize,
    fail_after: usize,
}

impl FailingReader {
    fn new(bytes: Vec<u8>, fail_after: usize) -> Self {
        Self {
            bytes,
            position: 0,
            fail_after,
        }
    }
}

impl Read for FailingReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.position >= self.fail_after {
            return Err(io::Error::other("synthetic read failure"));
        }
        let end = self
            .bytes
            .len()
            .min(self.fail_after)
            .min(self.position.saturating_add(output.len()));
        let count = end.saturating_sub(self.position);
        output[..count].copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        Ok(count)
    }
}

#[derive(Default)]
struct FailingWriter {
    bytes: Vec<u8>,
    remaining: usize,
}

impl FailingWriter {
    fn after(bytes: usize) -> Self {
        Self {
            bytes: Vec::new(),
            remaining: bytes,
        }
    }
}

impl Write for FailingWriter {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::other("synthetic write failure"));
        }
        let count = self.remaining.min(input.len());
        self.bytes.extend_from_slice(&input[..count]);
        self.remaining -= count;
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn valid_bytes() -> Vec<u8> {
    let mut header = [0_u8; CHUNK_SIZE];
    header[0] = 9;
    header[2..6].copy_from_slice(b"CMPD");
    let mut compound = [0_u8; CHUNK_SIZE];
    compound[0] = 1;
    [header, compound].into_iter().flatten().collect()
}

fn test_root(test_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("io-faults-{}-{test_name}", std::process::id()))
}

#[test]
fn reader_faults_preserve_the_underlying_io_source() {
    let bytes = valid_bytes();
    for fail_after in [0, 80, CHUNK_SIZE, CHUNK_SIZE + 44] {
        let error = RawDatabase::from_reader(FailingReader::new(bytes.clone(), fail_after))
            .expect_err("synthetic reader must fail");
        assert!(matches!(error, ParseError::Io(_)));
        assert!(error.source().is_some());
    }
}

#[test]
fn writer_fault_after_partial_output_preserves_the_io_source() {
    let database = RawDatabase::from_bytes(&valid_bytes()).unwrap();
    let mut writer = FailingWriter::after(19);
    let error = database
        .write_to(&mut writer)
        .expect_err("synthetic writer must fail");
    assert!(matches!(error, SerializeError::Io(_)));
    assert!(error.source().is_some());
    assert!(!writer.bytes.is_empty());
    assert!(writer.bytes.len() < CHUNK_SIZE);
}

#[test]
fn path_errors_include_paths_and_sources_without_touching_input_directories() {
    let root = test_root("paths");
    fs::create_dir_all(&root).unwrap();
    let missing = root.join("missing.CDB");
    let error = RawDatabase::from_path(&missing).expect_err("missing path must fail");
    match &error {
        ParseError::OpenPath { path, .. } => assert_eq!(path, &missing),
        other => panic!("expected path-aware open error, got {other:?}"),
    }
    assert!(error.source().is_some());

    let directory_error = RawDatabase::from_path(&root).expect_err("directory is not a CDB file");
    assert!(directory_error.source().is_some());

    let nul_path = Path::new("invalid\0path.CDB");
    let nul_error = RawDatabase::from_path(nul_path).expect_err("NUL path must fail");
    assert!(matches!(nul_error, ParseError::OpenPath { .. }));
    assert!(nul_error.source().is_some());

    let output_error = RawDatabase::from_bytes(&valid_bytes())
        .unwrap()
        .write_to_path(&root)
        .expect_err("directory cannot be replaced as a CDB file");
    assert!(matches!(output_error, SerializeError::CreatePath { .. }));
    assert!(output_error.source().is_some());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn windows_style_space_unicode_and_supported_long_paths_are_accepted() {
    let root = test_root("unicode").join("space and unicode Ω");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("synthetic.CDB");
    fs::write(&path, valid_bytes()).unwrap();
    assert_eq!(RawDatabase::from_path(&path).unwrap().chunks().len(), 2);

    let long_directory = root
        .join("a".repeat(70))
        .join("b".repeat(70))
        .join("c".repeat(70))
        .join("d".repeat(70));
    match fs::create_dir_all(&long_directory) {
        Ok(()) => {
            let long_path = long_directory.join("synthetic.CDB");
            fs::write(&long_path, valid_bytes()).unwrap();
            assert_eq!(
                RawDatabase::from_path(&long_path).unwrap().chunks().len(),
                2
            );
        }
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::InvalidInput | io::ErrorKind::NotFound | io::ErrorKind::Other
            ) => {}
        Err(error) => panic!("unexpected long-path setup error: {error}"),
    }
    fs::remove_dir_all(test_root("unicode")).unwrap();
}
