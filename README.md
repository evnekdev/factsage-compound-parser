# factsage-compound-parser

Native Rust parsing foundations and a read-only semantic model for FactSage Compound Database (.CDB) files.

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
- [docs/schema-validation.md](docs/schema-validation.md) - Kaitai validation results against the private local fixture.
- [schemas/factsage_compound.ksy](schemas/factsage_compound.ksy) - Kaitai Struct YAML schema.
- [src/raw](src/raw) - native lossless raw-record parser.
- [src/domain](src/domain) - read-only semantic grouping layer.

## Current status

The first two native Rust milestones are implemented:

- parses a flat sequence of exact 256-byte chunks;
- validates non-empty input, chunk alignment, first ID 9, and CMPD magic;
- decodes every known chunk ID into a typed raw representation;
- preserves unknown, reserved, padding, and fixed-width text bytes;
- preserves the original heat-capacity IDs 2 through 6;
- groups compounds, phases, CP ranges, kappa ranges, and comment fragments;
- links ranges to phases by exact raw phase ID;
- exposes phase state, index, compact labels, ChemApp labels, and safe text helpers;
- reports non-fatal semantic issues as typed diagnostics while retaining source records;
- exposes byte-slice, reader, and path-based raw and domain parsing APIs.

The raw API is lossless and flat. The domain API is a read-only view over that raw data: it adds ownership and lookup relationships but does not remove or rewrite raw records. Unknown chunks inside compound groups, orphan ranges, and ambiguous links remain accessible.

Invalid group ordering is fatal by default. Duplicate phase IDs, orphan or ambiguous ranges, questionable phase indexes, invalid ASCII, invalid temperature bounds, and unknown chunks inside a compound group are returned as typed diagnostics.

## Minimal domain example

~~~rust
use factsage_compound_parser::Database;

fn list_compounds(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::from_path(path)?;

    for compound in &database.compounds {
        let name = compound.name()?;
        let formula = compound.formula()?;
        println!("{name} ({formula})");
        for phase in &compound.phases {
            println!("  {}: {}", phase.chemapp_label(), phase.name()?);
        }
    }

    for diagnostic in &database.diagnostics {
        eprintln!("diagnostic at chunk {}: {:?}", diagnostic.chunk_index, diagnostic.kind);
    }
    Ok(())
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

Fixed-width text remains available in raw byte arrays. Domain ASCII accessors return an explicit decoding error; Windows-1252 comment decoding is infallible. Text helpers are convenience views and do not replace the raw representation.

## Unsupported functionality

Thermodynamic equation evaluation, heat-capacity integration, unit conversion, density interpretation, OLE date conversion, mutation, CDB writing, round-trip serialization, serde, CLI tools, Python bindings, and advanced kappa evaluation are not implemented.

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
