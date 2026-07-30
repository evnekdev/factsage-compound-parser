# Semantic domain model

The domain layer is a read-only grouping view built from `RawDatabase`. It does not reinterpret unknown fields, evaluate equations, convert units, or serialize files. Every typed record remains available through its raw field, and records that cannot be linked remain in explicit preservation collections.

## Grouping state machine

The parser consumes the flat chunk stream in this order:

~~~text
database header
compound
phase*
(CP | kappa)*
comment*
compound ... or end of file
~~~

The first chunk must be the raw database header. A compound may have no phases, ranges, or comments. Once a range has been seen, a later phase is an ordering error. Once a comment has been seen, a later phase or range is an ordering error. Known ordering errors return `DatabaseError::Domain` and do not produce a partial `Database`.

Unknown chunk IDs inside an active compound group are retained in `Compound::unknown_chunks` and produce an `UnknownChunk` diagnostic. An unknown ID where a new compound is required is a fatal ordering error.

## Phase IDs and labels

State decoding follows the established Python behaviour:

| Raw ID condition | State | Index |
| --- | --- | --- |
| greater than 990 | aqueous | raw ID minus 990 |
| greater than 900 | gas | raw ID minus 900 |
| greater than 800 | liquid | raw ID minus 800 |
| otherwise | solid | raw ID minus 100 |

The compact label uses uppercase state prefixes: `S1`, `L2`, `G2`, or `AQ1`. The ChemApp-style label uses lowercase prefixes and omits the suffix for index 1: `s`, `l2`, `g2`, or `aq`.

Zero and negative calculated indexes are preserved rather than rejected. They produce a `SuspiciousPhaseId` diagnostic.

## Range linking

After phases have been collected for a compound, CP and kappa records are linked by exact equality of `phase_id_raw`.

- CP IDs 2 through 6 retain their `HeatCapacityKind`.
- Kappa records become `PhysicalPropertyRange` values.
- A single matching phase receives the range in stream order.
- No matching phase produces an `OrphanRange` and an orphan diagnostic.
- Multiple matching phases produce an `OrphanRange` with `AmbiguousPhase` reason and an ambiguity diagnostic.

The original raw range record is nested inside each attached or orphan range. Non-finite temperature bounds and reversed bounds are diagnostics only.

## Comments and text

Comment chunks are consecutive fixed-width fragments. `Compound::joined_comment` decodes each fragment as Windows-1252, removes only storage padding, and concatenates fragments without inserting separators.

Compound names, formulae, and phase names are fixed-width ASCII-compatible fields. Their domain accessors return `Result<Cow<str>, TextDecodeError>` so invalid bytes are explicit. The raw byte arrays remain available regardless of decoding success.

## Diagnostics

`Database::diagnostics` contains typed, non-fatal issues with zero-based source chunk indexes, absolute byte offsets, and compound indexes where applicable. It currently covers duplicate phase IDs, orphan CP and kappa ranges, ambiguous links, invalid ASCII, suspicious phase IDs, non-finite or reversed temperature bounds, and unknown chunks inside compound groups.

Structural stream ordering is intentionally stricter than semantic validation: ordering errors are fatal, while questionable values and unresolved relationships are preserved and reported.
