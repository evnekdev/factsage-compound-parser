# Contributing

## Toolchain and setup

Use Rust stable for development. The minimum supported Rust version is 1.85.0 and the repository uses edition 2024. The repository also supports nightly only for cargo-fuzz target builds and sanitizer runs.

After cloning, run:

```text
cargo check --all-targets --all-features
```

The optional local `examples/MS16BASE.CDB` fixture is proprietary, ignored, and not required for setup or CI.

## Required checks

Before opening a pull request, run:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --doc
cargo check --examples
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo package --list
cargo package
cargo +1.85.0 check --all-targets --all-features
cargo +1.85.0 test --lib --all-features
```

Run synthetic Criterion benchmarks, bounded property tests, and fuzz smoke tests when a change affects parsing, grouping, serialization, editing, or thermodynamic validation. See [fuzzing](docs/fuzzing.md) for platform-specific commands.

Run private-fixture and Python-parity checks only when the local fixture and Python checkout are available. They must report aggregate values only.

## Format and test changes

The CDB layout is reverse-engineered. Changes to schemas, offsets, signedness, encodings, or semantic rules must cite validated source or fixture behavior in the change description. Add synthetic tests for new behavior; synthetic data must not be copied from a proprietary database.

Preserve the ownership model: `RawDatabase` is the sole owned physical representation, `DomainIndex` stores only indexes and diagnostics, and domain views borrow raw data. Do not introduce a second authoritative domain record collection, a file-backed parser, memory mapping, a piece table, or a linked-list chunk store without measured justification and an architecture review.

Never attach, stage, commit, copy, encode, upload, paste, benchmark, or use as a fuzz seed `MS16BASE.CDB` or its record contents. Use minimal synthetic reproductions in issues and pull requests.