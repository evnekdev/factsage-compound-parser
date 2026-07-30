# Fuzzing

The `fuzz/` Cargo project contains synthetic-seed-free `cargo-fuzz` targets:

- `raw_parse`: byte-slice and streaming raw parsing.
- `raw_roundtrip`: successful raw parses must serialize byte-for-byte.
- `domain_index`: index construction and borrowed traversal.
- `editor_operations`: structural insertion, lazy index rebuild, and serialization.
- `thermo_eval`: bounded safe-temperature density and CP access.

No proprietary CDB bytes may be added as a corpus, artifact, regression, or seed. Each target returns before parsing inputs larger than 1 MiB, so fuzzer work and allocator pressure remain bounded even when a runner is configured without a maximum input length.

Use a nightly toolchain on a libFuzzer-supported platform such as Linux or WSL:

```text
cargo +nightly fuzz run raw_parse -- -runs=1000
cargo +nightly fuzz run raw_roundtrip -- -runs=1000
cargo +nightly fuzz run domain_index -- -runs=1000
cargo +nightly fuzz run editor_operations -- -runs=1000
cargo +nightly fuzz run thermo_eval -- -runs=1000
```

The local Windows MSVC environment type-checks all targets with `cargo +nightly check --manifest-path fuzz/Cargo.toml --bins`, but cannot link libFuzzer sanitizer coverage because the required Clang sanitizer runtime is unavailable. Run sanitizer smoke tests in Linux/WSL or a CI runner with the appropriate runtime. This is an environment limitation, not a skipped parser test.

The bounded `proptest` integration suite complements fuzzing in normal Rust test runs. It covers raw round trips, floating-point bit patterns, unknown chunks, insertion/removal reversibility, setter locality, index rebuilding, and repeated view traversal.
