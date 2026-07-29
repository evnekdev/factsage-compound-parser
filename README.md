# factsage-compound-parser

Native Rust parsing foundations and binary-format documentation for FactSage Compound Database (.CDB) files.

The repository is derived from two earlier projects:

- evnekdev/factsage-compound: the completed Python/NumPy parser and semantic model.
- evnekdev/factsage-compound-docs: the unfinished explanatory wiki and initial Kaitai draft.

The validated Kaitai schema remains the primary physical-layout specification.

## Contents

- [docs/format-overview.md](docs/format-overview.md) - file organisation, chunk ordering, endianness, and parser invariants.
- [docs/chunk-layouts.md](docs/chunk-layouts.md) - byte-accurate layouts for every known 256-byte chunk.
- [docs/parsing-model.md](docs/parsing-model.md) - future reconstruction of compounds, phases, ranges, comments, and physical-property records.
- [docs/semantic-rules.md](docs/semantic-rules.md) - future unit conversion, phase identifiers, strings, dates, density encoding, and unresolved fields.
- [docs/schema-validation.md](docs/schema-validation.md) - Kaitai validation results against the private local fixture.
- [schemas/factsage_compound.ksy](schemas/factsage_compound.ksy) - Kaitai Struct YAML schema.
- [src/raw](src/raw) - native lossless raw-record parser.

## Current status

The first native Rust milestone is implemented:

- parses a flat sequence of exact 256-byte chunks;
- validates non-empty input, chunk alignment, first ID 9, and CMPD magic;
- decodes every known chunk ID into a typed raw representation;
- preserves unknown, reserved, padding, and fixed-width text bytes;
- preserves the original heat-capacity IDs 2 through 6;
- provides safe Windows-1252 comment decoding helpers;
- exposes byte-slice, reader, and path-based parsing APIs.

The high-level semantic model is intentionally not implemented yet. Compound ownership, phase/range linking, thermodynamic evaluation, unit conversion, OLE date conversion, and interpretation of unknown fields remain future work.

Writing and round-trip serialization are not implemented.

## Minimal parsing example

~~~rust
use factsage_compound_parser::{RawChunk, RawDatabase};

fn inspect(path: &std::path::Path) -> Result<(), factsage_compound_parser::ParseError> {
    let database = RawDatabase::from_path(path)?;

    for chunk in &database.chunks {
        match chunk {
            RawChunk::Compound(compound) => {
                println!("compound bytes: {}", compound.compound_name.len());
            }
            RawChunk::HeatCapacity { kind, chunk } => {
                println!("CP ID {} has {} coefficients", kind.id(), chunk.coefficients.len());
            }
            RawChunk::Unknown { id, body } => {
                println!("preserved unknown ID {} with {} body bytes", id, body.len());
            }
            _ => {}
        }
    }

    Ok(())
}
~~~

The parser keeps raw fields available for later reinterpretation. Text helpers are convenience views over the preserved bytes and do not replace the raw representation.

## Validation

The ignored local fixture examples/MS16BASE.CDB is used only when it exists locally. It is never required for public CI and must not be staged, copied, encoded, or uploaded.

Run the native tests with:

~~~text
cargo test --all-targets
~~~

Run the schema validator separately after generating its ignored Python output as described in docs/schema-validation.md.
