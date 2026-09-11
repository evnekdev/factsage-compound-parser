# factsage-compound-parser

A safe native Rust foundation for inspecting, grouping, evaluating established fields, editing selected fields, and losslessly serializing FactSage's shared Compound/Function Database (`.CDB`/`.FDB`) binary family.

Windows is the primary validated platform because FactSage and the reference databases are Windows-based. The Rust implementation remains portable where straightforward and CI also performs secondary Linux and macOS checks. The crate contains no proprietary CDB data.

## Independence

This is an independent, reverse-engineered open-source project. It is not
affiliated with, endorsed by, or supported by the FactSage developers or
distributors. FactSage is a trademark of its respective owners.

## Ownership and API layers

The crate uses owned raw parsing and zero-duplication semantic views:

- `raw::RawDatabase` owns the contiguous authoritative physical chunk stream. It preserves known and unknown chunks, reserved fields, padding, text bytes, and original CP IDs 2 through 6.
- `domain::DomainIndex` stores only chunk indexes, semantic relationships, and diagnostics for one raw-stream generation.
- `domain::DatabaseView<'_>` borrows matching raw storage and an index to expose header, compounds, phases, CP ranges, structurally linked ID-11 records, comments, and diagnostics without cloning raw records.
- `domain::Database` is a read-only owner of one raw stream plus its index. Call `Database::view` for semantic traversal.
- `edit::DatabaseEditor` owns one mutable raw stream, lazily rebuilds its index after structural changes, and returns borrowed semantic views.
- `thermo` exposes established unit conversion, OLE Automation dates, provisional density decoding, stored CP-expression evaluation, and evidence-backed FDB ordinary-phase thermodynamic views.

`RawDatabase` is the only serialization authority. Low-level raw insertion or removal can create a temporarily invalid semantic stream; rebuilding an index validates grouping. Path APIs accept `AsRef<Path>` and preserve the path plus underlying OS error for open/read/write failures.

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

Evaluate stored CP for the first phase of the first compound. The API derives the compound energy unit and reports gaps or true overlaps as typed errors.

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

## Lossless raw serialization and controlled edits

For unmodified input accepted by the raw parser:

```text
input == RawDatabase::from_bytes(input)?.to_bytes()?
```

The guarantee covers chunk order and IDs, unknown chunks, reserved/padding bytes, fixed-width text, and parsed IEEE-754 bit patterns. `from_reader` consumes exact 256-byte records without retaining a second full-file input buffer; `write_to` streams records without creating a full output vector.

The editor supports only well-established fields. Names require strict ASCII and finite numeric setters retain a valid index. Structural raw edits invalidate the cached index. The editor deliberately does not change CP anchors when ordinary phase enthalpy or entropy changes, because the anchor convention remains unresolved.

A controlled edit can be written to a caller-selected destination:

```rust
use std::error::Error;
use std::path::Path;
use factsage_compound_parser::edit::DatabaseEditor;

fn rename_first_compound(input: &Path, output: &Path) -> Result<(), Box<dyn Error>> {
    let mut editor = DatabaseEditor::from_path(input)?;
    editor.set_compound_name(0, "Example")?;
    editor.write_to_path(output)?;
    Ok(())
}
```

ID-11 records are structurally parsed, preserved, and linked by exact raw phase ID within their compound. Their physical equation, units, and coefficient meanings are not established and are not evaluated.

## FDB provider semantics

`.FDB` is a logical Function Database role built on the same `CMPD` physical
family as `.CDB`; the filename is not provider-semantic proof. The parser's
`CompoundDatabaseProfileEvidence` checks the observed FDB-compatible header
guardrail (`read_flag == 0`), but valid CDB files can satisfy it too. A caller
must supply the logical bundle role and accept that the result is compatibility
evidence, not an intrinsic CDB/FDB classification.

For a validated ordinary FDB phase, each CP range supplies H and S constants at
298.15 K plus `Cp(T) = sum(c_i T^p_i)`. The explicitly named
`fdb_phase_thermodynamic_view` API returns `PhaseThermodynamicView::Ordinary`,
preserves source order, requires finite positive contiguous ranges of one CP
kind, and refuses gaps, overlaps, mixed kinds, and extrapolation. It exposes
provider data only; downstream code owns canonical Gibbs conversion. ID-8
transition parent links are typed, but transition effective-G semantics remain
pending independent evidence.

## Windows validation and private-data policy

The ignored local `examples/MS16BASE.CDB` fixture and the installed FactSage corpus are proprietary. They are never staged, copied, encoded, uploaded, printed, used as fuzz seeds, or modified. Corpus validation opens source files read-only, canonicalizes every path under the configured root, serializes only to memory, and writes its aggregate local report only to `target/factsage-corpus-report.txt`.

```powershell
$env:FACTSAGE_FACTDATA_ROOT = 'C:\path\to\FACTDATA'
cargo test --test factsage_corpus -- --ignored --nocapture
```

The latest local Windows validation scanned 13 `.CDB` candidates (15,345,664 bytes): 12 parsed Compound Databases, all with exact in-memory round trips and deterministic domain indexes. The remaining candidate begins with ID 0 rather than the required Compound Database header and is reported as an unsupported format candidate, not reinterpreted.

## Release status

**Recommendation: Ready to publish experimental 0.1.0.**

This is not a production-readiness claim. The implementation has broad Windows corpus validation, lossless round trips, typed I/O errors, synthetic corruption tests, and complete Rustdoc. Production use still requires acceptance of reverse-engineered limitations: ID-11 physics, kappa units, CP integration conventions, density units, unknown fields, and extended fuzzing remain unresolved.

## API stability

This is an experimental `0.1.x` library. Public APIs may evolve, but breaking
changes are documented explicitly and patch releases should not silently add
avoidable breaking changes. The reviewed [0.1.0 public API baseline](api/0.1.0.txt)
and its [comparison workflow](api/README.md) support future compatibility checks;
this project does not promise `1.0` stability yet.

## Documentation

- [Architecture and API migration](docs/architecture.md)
- [Windows and synthetic performance measurements](docs/performance.md)
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

## Licensing and toolchain

This crate is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE). The declared minimum supported Rust version is 1.85.0, required for edition 2024 and verified on Windows.

GitHub Actions makes Windows stable the primary quality job: formatting, Clippy, tests, doctests, examples, strict rustdoc, and packaging. Windows Rust 1.85.0 validates the MSRV. Linux/macOS checks remain secondary portability coverage; a scheduled security workflow runs `cargo audit` and pull requests receive dependency review.
