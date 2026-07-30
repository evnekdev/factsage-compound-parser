# Contributing

## Toolchain and support policy

Windows is the primary validated platform because FactSage and the reference databases are Windows-based. The crate is portable Rust and Linux/macOS remain supported where straightforward, but Windows validation takes priority for release decisions.

Use Rust stable for development. The minimum supported Rust version is 1.85.0 and the repository uses edition 2024. Nightly is optional for cargo-fuzz target builds and sanitizer runs.

## Required checks

Before opening a pull request, run:

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --doc
cargo check --examples
$env:RUSTDOCFLAGS = '-D warnings'; cargo doc --no-deps --all-features
cargo package --list
cargo package
cargo audit
cargo +1.85.0 check --all-targets --all-features
cargo +1.85.0 test --lib --all-features
```

Run synthetic Criterion benchmarks, bounded property tests, and fuzz smoke tests when a change affects parsing, grouping, serialization, editing, or thermodynamic validation. See [fuzzing](docs/fuzzing.md) for platform-specific commands.

## Private fixtures and the local FactSage corpus

The optional repository-local `examples/MS16BASE.CDB` fixture and configured FactSage installation corpus are proprietary. They are not required for public CI.

When the corpus is available, run only the ignored read-only validator:

```powershell
$env:FACTSAGE_FACTDATA_ROOT = 'C:\path\to\FACTDATA'
cargo test --test factsage_corpus -- --ignored --nocapture
```

Never edit, rename, delete, move, copy, encode, upload, print, benchmark from persisted copies of, or use as fuzz seeds any proprietary CDB. The scanner opens corpus files read-only, validates canonical containment beneath the configured root, and writes only an aggregate report under ignored `target/`.

## Format and API changes

The CDB layout is reverse-engineered. Changes to schemas, offsets, signedness, encodings, or semantic rules must cite validated source or aggregate fixture behavior in the change description. Add synthetic tests for new behavior; synthetic data must not be copied from proprietary databases.

Preserve the ownership model: `RawDatabase` is the sole owned physical representation, `DomainIndex` stores only indexes and diagnostics, and domain views borrow raw data. Do not introduce a second authoritative domain record collection, a file-backed parser, memory mapping, a piece table, or linked-list chunk storage without measured justification and an architecture review.

ID-11 records are structurally supported and linked by raw phase ID. Do not add equations, units, or semantic meanings for their coefficient arrays without independent evidence.