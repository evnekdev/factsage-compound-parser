# factsage-compound-parser

Knowledge base and binary-format specification for implementing a Rust parser for FactSage Compound Database (`.CDB`) files.

This repository is derived from two earlier projects:

- `evnekdev/factsage-compound`: the completed Python/NumPy parser and semantic model.
- `evnekdev/factsage-compound-docs`: the unfinished explanatory wiki and initial Kaitai draft.

The Python implementation is treated as the executable reference whenever it disagrees with the older wiki draft.

## Contents

- [`docs/format-overview.md`](docs/format-overview.md) — file organisation, chunk ordering, endianness, and parser invariants.
- [`docs/chunk-layouts.md`](docs/chunk-layouts.md) — byte-accurate layouts for every known 256-byte chunk.
- [`docs/parsing-model.md`](docs/parsing-model.md) — reconstruction of compounds, phases, ranges, comments, and physical-property records.
- [`docs/semantic-rules.md`](docs/semantic-rules.md) — unit conversion, phase identifiers, strings, dates, density encoding, and unresolved fields.
- [`schemas/factsage_compound.ksy`](schemas/factsage_compound.ksy) — Kaitai Struct YAML schema for the binary layout.

## Kaitai Struct

[Kaitai Struct](https://kaitai.io/) is a declarative language and compiler for binary formats. A `.ksy` YAML file describes byte order, primitive fields, arrays, conditional/switch-based records, nested types, and validation rules. The Kaitai compiler can turn the schema into parsers for Rust and many other languages, and the Kaitai visualizer can inspect a `.CDB` file against the same schema.

The schema in this repository models the **physical binary layout**. It deliberately does not attempt to reconstruct the higher-level compound hierarchy inside Kaitai; that association logic belongs in the Rust domain layer and is documented separately.

## Status

The layout is reconstructed from the finished Python parser. Known unknown/reserved fields are preserved as raw bytes. Before publishing a Rust crate, validate the schema and parser against representative official and user-created `.CDB` files, especially files containing transition phases, all five heat-capacity chunk IDs, aqueous phases, and extended physical-property chunks.
