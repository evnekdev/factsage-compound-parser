# Contributing

## Toolchain and setup

Use Rust stable for development. The minimum supported Rust version is 1.85.0; the repository uses edition 2024.

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
```

Run the private-fixture and Python parity checks only when the local fixture and Python checkout are available. They must report aggregate values only.

## Format and test changes

The CDB layout is reverse-engineered. Changes to schemas, offsets, signedness, encodings, or semantic rules must cite the validated source or fixture behaviour in the change description. Add synthetic tests for new behaviour; synthetic data must not be copied from a proprietary database.

Never attach, stage, commit, copy, encode, upload, or paste `MS16BASE.CDB` or its record contents. Use minimal synthetic reproductions in issues and pull requests.
