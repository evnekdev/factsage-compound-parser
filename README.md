# factsage-compound-parser

A safe native Rust foundation for inspecting, grouping, evaluating, editing selected fields, and losslessly serializing FactSage Compound Database (`.CDB`) files.

The crate is based on a validated Kaitai layout, the completed companion Python parser, and aggregate-only validation against a local private fixture. It does not contain proprietary CDB data.

## API layers

- `raw::RawDatabase` is the authoritative flat physical stream. It preserves all known and unknown chunks, reserved fields, padding, text bytes, and original CP IDs 2 through 6. It parses and writes CDB bytes losslessly.
- `domain::Database` is a read-only grouped view: header, compounds, phases, CP/kappa ranges, comments, and typed diagnostics.
- `thermo` exposes established unit conversion, OLE Automation dates, provisional density decoding, and stored CP-expression evaluation.
- `edit::DatabaseEditor` owns a mutable raw stream for controlled edits, then rebuilds a domain view on demand.

`RawDatabase` remains the only serialization authority. A grouped `Database` never silently rewrites or reorders records.

## Minimal examples

Parse and group a database:

```rust
use factsage_compound_parser::domain::Database;

fn list_phases(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::from_path(path)?;
    for compound in &database.compounds {
        println!("{}", compound.name()?);
        for phase in &compound.phases {
            println!("  {}: {}", phase.chemapp_label(), phase.name()?);
        }
    }
    Ok(())
}
```

Evaluate stored CP for a selected phase. The model uses the compound's established energy-unit code and reports gaps or true overlaps as typed errors.

```rust
use factsage_compound_parser::domain::Database;

fn heat_capacity(path: &std::path::Path) -> Result<f64, Box<dyn std::error::Error>> {
    let database = Database::from_path(path)?;
    let compound = database`n        .compounds`n        .first()`n        .ok_or_else(|| std::io::Error::other("database has no compounds"))?;
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

The guarantee covers original chunk order and IDs, unknown chunks, reserved/padding bytes, fixed-width text, and parsed IEEE-754 bit patterns. The private fixture is checked entirely in memory; no output copy is created.

See [serialization](docs/serialization.md) and [editing](docs/editing.md) for the authoritative-data and setter policies.

## Diagnostics and safe limits

The domain model treats invalid stream ordering as fatal. It retains and reports non-fatal issues such as unknown chunks inside a compound group, duplicate phase IDs, ambiguous/missing range links, invalid ASCII, questionable phase indexes, and CP bound problems.

The editor intentionally supports only well-established fields. Names must fit their strict ASCII fixed-width destination; setters reject non-finite numeric values and unknown energy units. It does not currently edit formulae, comments, density, CP expressions, kappa records, reserved fields, or unknown chunks.

The Python project's ordinary-phase setters propagate values into CP anchor fields, but this crate does not: the anchor convention is unresolved. Rust also corrects the Python transition-enthalpy setter's unit-conversion asymmetry.

## Format knowledge and unsupported semantics

Implemented read-only thermo features include calorie/joule handling, OLE Automation date conversion, provisional density remainder decoding, and eight-term CP evaluation. CP integration, arbitrary-temperature enthalpy/entropy/Gibbs calculations, phase-transition path traversal, advanced kappa evaluation, density units, mutation outside the documented editor fields, and all inferred meanings for unknown data remain out of scope.

## Documentation

- [Format overview](docs/format-overview.md)
- [Chunk layouts](docs/chunk-layouts.md)
- [Parsing model](docs/parsing-model.md)
- [Domain model](docs/domain-model.md)
- [Thermodynamic semantics](docs/thermodynamic-semantics.md)
- [Raw serialization](docs/serialization.md)
- [Controlled editing](docs/editing.md)
- [Python parity](docs/python-parity.md)
- [Validation status](docs/validation-status.md)
- [Release-readiness review](docs/release-readiness.md)

## Validation and fixture policy

`examples/MS16BASE.CDB` is proprietary, intentionally ignored, and optional. Tests skip cleanly when it is unavailable. It must never be staged, copied, encoded, uploaded, or printed.

Run the public checks:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo doc --no-deps
```

For the optional aggregate-only Python parity check, see [python parity](docs/python-parity.md).