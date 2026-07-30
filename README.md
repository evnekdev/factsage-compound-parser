# factsage-compound-parser

A safe native Rust foundation for inspecting, grouping, evaluating, editing selected fields, and losslessly serializing FactSage Compound Database (`.CDB`) files.

The crate is based on a validated Kaitai layout, the companion Python parser, and aggregate-only validation against a local private fixture. It does not contain proprietary CDB data.

## Ownership and API layers

The crate uses owned raw parsing and zero-duplication semantic views:

- `raw::RawDatabase` owns the contiguous, authoritative physical chunk stream. It preserves known and unknown chunks, reserved fields, padding, text bytes, and original CP IDs 2 through 6.
- `domain::DomainIndex` stores only chunk indexes, semantic relationships, and diagnostics for one raw-stream generation.
- `domain::DatabaseView<'_>` borrows a matching `RawDatabase` and `DomainIndex` to expose header, compounds, phases, CP/kappa ranges, comments, and diagnostics without cloning raw records.
- `domain::Database` is a read-only owner of one raw stream and its index. Call `Database::view` for semantic traversal.
- `edit::DatabaseEditor` owns one mutable raw stream, lazily rebuilds its index after structural changes, and returns borrowed semantic views.
- `thermo` exposes established unit conversion, OLE Automation dates, provisional density decoding, and stored CP-expression evaluation.

`RawDatabase` is the only serialization authority. Low-level raw insertion or removal can create a temporarily invalid semantic stream; rebuild an index or request an editor view to validate ordering.

## Minimal examples

Parse, index, and traverse a database:

```rust
use factsage_compound_parser::domain::Database;

fn list_phases(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::from_path(path)?;
    let view = database.view()?;
    for compound in view.compounds() {
        println!("{}", compound.name()?);
        for phase in compound.phases() {
            println!("  {}: {}", phase.chemapp_label(), phase.name()?);
        }
    }
    Ok(())
}
```

Evaluate stored CP for the first phase of the first compound. The API uses the compound's established energy-unit code and reports gaps or true overlaps as typed errors.

```rust
use factsage_compound_parser::domain::Database;

fn heat_capacity(path: &std::path::Path) -> Result<f64, Box<dyn std::error::Error>> {
    let database = Database::from_path(path)?;
    let compound = database
        .view()?
        .compounds()
        .next()
        .ok_or_else(|| std::io::Error::other("database has no compounds"))?;
    Ok(compound.heat_capacity_at(0, 1000.0)?)
}
```

Edit a documented field and serialize to a caller-selected output path:

```rust
use factsage_compound_parser::edit::DatabaseEditor;

fn rename(path: &std::path::Path, output: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = DatabaseEditor::from_path(path)?;
    editor.set_compound_name(0, "Example")?;
    editor.write_to_path(output)?;
    Ok(())
}
```

## Lossless raw serialization

For unmodified input accepted by the raw parser:

```text
input == RawDatabase::from_bytes(input)?.to_bytes()?
```

The guarantee covers original chunk order and IDs, unknown chunks, reserved/padding bytes, fixed-width text, and parsed IEEE-754 bit patterns. `from_reader` reads exact 256-byte records without retaining a second full-file buffer; `write_to` streams records without building a full output buffer.

## Diagnostics and editing policy

Invalid known-chunk ordering is fatal during index construction. The index retains non-fatal issues such as unknown chunks inside a compound group, duplicate phase IDs, ambiguous/missing range links, invalid ASCII, questionable phase indexes, and CP-bound problems.

The editor supports only well-established fields. Names must fit strict ASCII fixed-width destinations; setters reject non-finite numeric values and unknown energy units. Structural raw edits invalidate the cached index. The editor deliberately does not change CP anchors when ordinary phase enthalpy or entropy changes, because the anchor convention remains unresolved.

## Format knowledge and unsupported semantics

Implemented read-only thermo features include calorie/joule handling, OLE Automation date conversion, provisional density remainder decoding, and eight-term CP evaluation. CP integration, arbitrary-temperature enthalpy/entropy/Gibbs calculations, phase-transition path traversal, advanced kappa evaluation, density units, mutation outside the documented editor fields, and all inferred meanings for unknown data remain out of scope.

## Documentation

- [Architecture and API migration](docs/architecture.md)
- [Performance measurements](docs/performance.md)
- [Fuzzing](docs/fuzzing.md)
- [Format overview](docs/format-overview.md)
- [Chunk layouts](docs/chunk-layouts.md)
- [Domain model](docs/domain-model.md)
- [Thermodynamic semantics](docs/thermodynamic-semantics.md)
- [Raw serialization](docs/serialization.md)
- [Controlled editing](docs/editing.md)
- [Python parity](docs/python-parity.md)
- [Validation status](docs/validation-status.md)
- [Release-readiness review](docs/release-readiness.md)

## Validation and fixture policy

`examples/MS16BASE.CDB` is proprietary, intentionally ignored, and optional. Tests skip cleanly when it is unavailable. It must never be staged, copied, encoded, uploaded, printed, used as a fuzz seed, or used as a benchmark input.

Run the public checks:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --doc
cargo check --examples
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo package
```

## Licensing and toolchain

This crate is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE). The declared minimum supported Rust version is 1.85.0, required for edition 2024 and verified with the MSRV checks.

GitHub Actions runs formatting, Clippy, tests, documentation tests, example checks, rustdoc warnings, and packaging on stable Linux; library checks on Rust 1.85.0; and all-target tests on Windows and macOS. Dependabot tracks Cargo and GitHub Actions updates. A scheduled security workflow runs `cargo audit` and dependency review runs for pull requests.

The crate is ready for a robustness-testing pass, not a production-ready release. The property suite and fuzz targets now provide a foundation, but longer Linux sanitizer runs, broader malformed-input corpus work, and a final API-stability review remain necessary.