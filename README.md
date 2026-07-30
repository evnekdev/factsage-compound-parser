# factsage-compound-parser

Native Rust parsing foundations, a read-only semantic model, and read-only thermodynamic accessors for FactSage Compound Database (.CDB) files.

The repository is derived from two earlier projects:

- evnekdev/factsage-compound: the completed Python/NumPy parser and semantic model.
- evnekdev/factsage-compound-docs: the explanatory wiki and initial Kaitai draft.

The validated Kaitai schema remains the primary physical-layout specification.

## Contents

- [docs/format-overview.md](docs/format-overview.md) - file organisation, chunk ordering, endianness, and parser invariants.
- [docs/chunk-layouts.md](docs/chunk-layouts.md) - byte-accurate layouts for every known 256-byte chunk.
- [docs/parsing-model.md](docs/parsing-model.md) - physical and semantic stream reconstruction.
- [docs/semantic-rules.md](docs/semantic-rules.md) - phase identifiers, strings, dates, density encoding, and unresolved fields.
- [docs/domain-model.md](docs/domain-model.md) - domain grouping, range linking, labels, and diagnostics.
- [docs/thermodynamic-semantics.md](docs/thermodynamic-semantics.md) - units, dates, density, CP evaluation, and boundaries.
- [docs/schema-validation.md](docs/schema-validation.md) - Kaitai validation results against the private local fixture.
- [schemas/factsage_compound.ksy](schemas/factsage_compound.ksy) - Kaitai Struct YAML schema.
- [src/raw](src/raw) - native lossless raw-record parser.
- [src/domain](src/domain) - read-only semantic grouping layer.
- [src/thermo](src/thermo) - read-only thermodynamic decoding and CP evaluation.

## Current status

The first three native Rust milestones are implemented:

- parses a flat sequence of exact 256-byte chunks;
- validates non-empty input, chunk alignment, first ID 9, and CMPD magic;
- decodes every known chunk ID into a typed raw representation;
- preserves unknown, reserved, padding, and fixed-width text bytes;
- preserves the original heat-capacity IDs 2 through 6;
- groups compounds, phases, CP ranges, kappa ranges, and comment fragments;
- links ranges to phases by exact raw phase ID;
- exposes phase state, index, compact labels, ChemApp labels, and safe text helpers;
- decodes compound energy and pressure unit codes;
- converts established calorie-based values to SI joules without mutating raw fields;
- converts OLE Automation dates to SystemTime;
- decodes provisional packed phase density values;
- evaluates stored eight-term CP expressions with inclusive range selection;
- reports non-fatal semantic issues as typed diagnostics while retaining source records;
- exposes byte-slice, reader, and path-based raw, domain, and thermodynamic APIs.

The raw API is lossless and flat. The domain API is a read-only view over that raw data: it adds ownership and lookup relationships but does not remove or rewrite raw records. The thermo API adds calculated views over preserved values; it does not cache or mutate them.

Invalid group ordering is fatal by default. Duplicate phase IDs, orphan or ambiguous ranges, questionable phase indexes, invalid ASCII, invalid temperature bounds, and unknown chunks inside a compound group are returned as typed diagnostics. Thermodynamic evaluation returns typed errors for invalid units, temperatures, ranges, coefficients, powers, dates, and density values.

## Minimal domain example

~~~rust
use factsage_compound_parser::Database;

fn list_compounds(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::from_path(path)?;
    for compound in &database.compounds {
        println!("{} ({})", compound.name()?, compound.formula()?);
        for phase in &compound.phases {
            println!("  {}: {}", phase.chemapp_label(), phase.name()?);
        }
    }
    Ok(())
}
~~~

## Thermodynamic example

This evaluates a stored CP expression for a selected phase. The chosen phase and temperature are application inputs; no proprietary names or values are required.

~~~rust
use factsage_compound_parser::Database;

fn evaluate_first_phase(path: &std::path::Path) -> Result<f64, Box<dyn std::error::Error>> {
    let database = Database::from_path(path)?;
    let compound = database.compounds.first().ok_or_else(|| std::io::Error::other("database has no compounds"))?;
    let temperature_k = 1000.0;
    Ok(compound.heat_capacity_at(0, temperature_k)?)
}
~~~

## Raw API

Use RawDatabase when the flat physical record stream and every preserved byte are the primary concern:

~~~rust
use factsage_compound_parser::{RawChunk, RawDatabase};

fn inspect(path: &std::path::Path) -> Result<(), factsage_compound_parser::ParseError> {
    let database = RawDatabase::from_path(path)?;
    for chunk in &database.chunks {
        if let RawChunk::Unknown { id, body } = chunk {
            println!("preserved unknown ID {id} with {} body bytes", body.len());
        }
    }
    Ok(())
}
~~~

Fixed-width text remains available in raw byte arrays. Domain ASCII accessors return an explicit decoding error; Windows-1252 comment decoding is infallible. Text helpers and thermodynamic accessors are convenience views and do not replace the raw representation.

## Unsupported functionality

Integrated enthalpy or entropy calculations, Gibbs-energy calculations, phase-transition path traversal, advanced kappa-property evaluation, pressure conversion, density setters, mutation, CDB writing, round-trip serialization, serde, CLI tools, Python bindings, and crates.io publication are not implemented.

## Validation

The ignored local fixture examples/MS16BASE.CDB is used only when it exists locally. It is never required for public CI and must not be staged, copied, encoded, or uploaded.

Run the native checks with:

~~~text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo doc --no-deps
~~~

Run the schema validator separately after generating its ignored Python output as described in docs/schema-validation.md.