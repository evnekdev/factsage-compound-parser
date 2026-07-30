# Raw serialization

`raw::RawDatabase` is the authoritative physical representation for CDB serialization. It retains the original flat chunk sequence and exposes:

```rust
let raw = factsage_compound_parser::raw::RawDatabase::from_path(path)?;
let bytes = raw.to_bytes()?;
raw.write_to_path(output_path)?;
```

`write_to_path` only writes to the path supplied by the caller. The crate never overwrites an input CDB path implicitly.

## Round-trip guarantee

For an unmodified byte stream accepted by `RawDatabase::from_bytes`, this invariant is tested:

```text
input == RawDatabase::from_bytes(input)?.to_bytes()?
```

The guarantee includes chunk ordering and IDs, CP IDs 2 through 6, fixed-width text, unknown chunk bodies, reserved fields, padding, and all little-endian numeric bytes. Parsed `f32` and `f64` values are re-emitted with `to_le_bytes`; synthetic tests cover infinities and non-canonical NaN payloads. No floating-point arithmetic occurs during an unmodified serialization.

The private local fixture is checked in memory only. No validation output is written beside it.

## Domain model boundary

`domain::Database` is a grouped read-only view. It intentionally does not serialize because grouping separates chunks into compound-owned collections and is not a second authoritative copy of the physical stream. Use `RawDatabase` for inspection and serialization, or `edit::DatabaseEditor` for controlled edits followed by serialization.

Unknown chunks remain serializable even when the domain view reports them as diagnostics.
