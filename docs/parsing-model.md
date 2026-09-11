# Parsing and ownership model

## Two parsing layers

The Rust implementation should separate:

1. **Binary layer** — reads an exact 256-byte chunk and decodes the fields selected by its ID.
2. **Domain layer** — groups chunks into compounds, links ranges to phases, decodes units and labels, and reports semantic inconsistencies.

This split mirrors the difference between the Kaitai schema and the completed Python classes.

## Forward parser state machine

After reading the database header, parse the remainder with a cursor:

```text
expect compound
  -> collect phases
  -> collect ranges
  -> collect comments
  -> expect compound or EOF
```

Pseudocode:

```rust
read header chunk (must be ID 9)
while !eof {
    let compound = read chunk (must be ID 1)

    while next ID is 7 or 8 {
        compound.phases.push(read phase)
    }

    while next ID is 2..=6 or 11 {
        let range = read range
        attach range to phase with matching phase_id_raw
    }

    while next ID is 10 {
        compound.comment_fragments.push(read comment)
    }

    output compound
}
```

Do not silently scan forward when an unexpected chunk appears. The Python parser raises if a compound was expected and another ID is found. Rust errors should include chunk index, byte offset, encountered ID, and expected category.

## Suggested raw model

```rust
struct RawDatabase {
    header: DatabaseHeaderChunk,
    chunks: Vec<Chunk>,
}

enum Chunk {
    Compound(CompoundChunk),
    Cp { kind: CpChunkKind, value: CpChunk },
    PhaseOrdinary(PhaseOrdinaryChunk),
    PhaseTransition(PhaseTransitionChunk),
    Comment(CommentChunk),
    Kappa(KappaChunk),
    Unknown { id: u8, body: [u8; 255] },
}
```

Preserving unknown chunks is preferable to rejecting them in the binary layer. The strict domain parser can then decide whether an unknown record makes a compound group uninterpretable.

## Suggested domain model

```rust
struct Database {
    metadata: DatabaseMetadata,
    compounds: Vec<Compound>,
}

struct Compound {
    header: EntryHeader,
    name: String,
    formula: String,
    units: Units,
    real_stoichiometry: [f64; 7],
    phases: Vec<Phase>,
    comment_fragments: Vec<String>,
    raw_reserved: CompoundReserved,
}

enum PhaseDefinition {
    Ordinary { enthalpy_298: f64, entropy_298: f64 },
    Transition {
        transition_enthalpy: f64,
        transition_temperature: f64,
        parent_phase_id_raw: i32,
    },
}

struct Phase {
    phase_id_raw: i32,
    definition: PhaseDefinition,
    name: String,
    density_raw: f64,
    heat_capacity_ranges: Vec<HeatCapacityRange>,
    physical_property_ranges: Vec<PhysicalPropertyRange>,
}
```

Keep the raw fields accessible, either directly or through a `raw` substructure. Reverse-engineered formats often need later reinterpretation without reparsing the original file.

## Linking ranges to phases

The Python parser attaches CP and kappa chunks by exact equality of `phase_id_raw` within the current compound. It does not use array position.

Recommended rules:

- Build a temporary `HashMap<i32, usize>` from phase ID to phase index after reading phases.
- Reject or warn on duplicate phase IDs in one compound.
- Attach each range to the matching phase.
- Report an orphan range when no phase matches.
- Preserve orphan records in a diagnostic collection rather than dropping bytes.

An older unused Python method additionally compared element IDs, integer coefficients and charge. The active parser only matches `phase_id_raw`; a Rust strict-validation option may compare the shared formula header as an additional consistency check.

For provider thermodynamic consumption, `CompoundView::fdb_phase_thermodynamic_view`
adds a narrower validation layer after this structural association. It retains
the CP source order and accepts an ordinary effective-G definition only when
all records use one CP kind, every range is finite and strictly positive, and
adjacent ranges share their exact stored boundary. It does not sort, merge,
bridge, or extrapolate records. The lower-level domain view remains lossless
and continues to expose malformed/orphan data for inspection.

## CP chunk IDs

IDs `2`, `3`, `4`, `5`, and `6` share one layout. Preserve the ID in an enum rather than normalising it away:

```rust
enum CpChunkKind {
    Type1, // ID 2
    Type2, // ID 4
    Type3, // ID 5
    Type4, // ID 3
    Type5, // ID 6
}
```

The unusual numbering above reproduces the names used by the Python constants. Since their semantic differences are unknown, public API names such as `Id2` through `Id6` may be less misleading.

The locally examined FDB corpus used one CP kind per phase. ID 2 supplied
multi-range sequences; IDs 4 and 5 appeared as single ranges. This is
validation evidence for the current provider view, not a universal semantic
meaning for the five IDs.

## Comments

ID-10 chunks are consecutive 80-byte fragments. Keep both:

- the individual fragments, for byte-preserving round trips;
- a convenience joined comment, formed in stream order.

Do not add separators unless sample files demonstrate that FactSage treats fragments as separate lines. A conservative join is direct concatenation after removing only storage padding.

## Error model

Useful error variants include:

- file length is not divisible by 256;
- empty file;
- first chunk is not ID 9;
- invalid `CMPD` magic;
- unexpected chunk category in the group state machine;
- duplicate phase ID;
- range references a missing phase;
- invalid ASCII under strict decoding;
- non-finite or implausible temperature bounds;
- lower temperature exceeds upper temperature.

Structural errors should be distinct from validation warnings so partially understood databases can still be inspected.

## Round-trip considerations

The Python library can modify its NumPy-backed fields and write the complete chunk array back. To preserve that capability in Rust:

- retain all unknown and padding bytes;
- avoid regenerating reserved strings;
- preserve original chunk order and CP IDs;
- define setters that update only the exact documented byte fields;
- test `parse -> serialize` for byte-for-byte equality before exposing editing.
