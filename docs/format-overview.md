# FactSage Compound Database format overview

## Physical organisation

A Compound Database (`.CDB`) file is a flat sequence of fixed-size **256-byte chunks**. There is no variable-length framing and no separate length field.

Each chunk consists of:

| Offset | Size | Meaning |
|---:|---:|---|
| 0 | 1 | Chunk ID (`u8`) |
| 1 | 255 | Chunk body selected by the ID |

The file length should therefore be divisible by 256. The first chunk is the database-header chunk (`ID 9`). All remaining chunks form consecutive compound groups.

All multi-byte numeric fields observed by the Python implementation are little-endian. NumPy used native-endian scalar types on Windows/x86, which is little-endian; the Kaitai schema makes this explicit with `endian: le`.

## Known chunk IDs

| ID | Meaning | Body type |
|---:|---|---|
| 1 | Compound definition | `compound_body` |
| 2 | Heat-capacity range variant 1 | `cp_body` |
| 3 | Heat-capacity range variant 4 | `cp_body` |
| 4 | Heat-capacity range variant 2 | `cp_body` |
| 5 | Heat-capacity range variant 3 | `cp_body` |
| 6 | Heat-capacity range variant 5 | `cp_body` |
| 7 | Ordinary phase: H298 and S298 | `phase_ordinary_body` |
| 8 | Transition phase: transition enthalpy and temperature | `phase_transition_body` |
| 9 | Database header | `database_header_body` |
| 10 | Compound comment fragment | `comment_body` |
| 11 | Extended physical-property / kappa range | `kappa_body` |

The five heat-capacity IDs share the same physical layout. Their exact semantic distinction is not established in the Python project and must not be invented by the Rust parser.

## Compound-group ordering

The reference Python parser relies on the following stream grammar:

```text
database_header
compound_group*

compound_group :=
    compound
    phase*
    (cp_range | kappa_range)*
    comment*
```

The next compound chunk starts the next group. A parser can therefore process the file in one forward pass:

1. Read and validate the header chunk.
2. Require a compound chunk.
3. Consume consecutive phase chunks (`7` or `8`).
4. Consume consecutive range chunks (`2` through `6`, or `11`).
5. Consume consecutive comment chunks (`10`).
6. Repeat until end of file.

This ordering is a semantic invariant of the completed Python parser, not a constraint currently enforced by the Kaitai schema. Kaitai exposes the raw chunk sequence; the Rust domain layer should reconstruct groups and report ordering errors clearly.

## Shared record header

Every non-database body starts with the same 31-byte header:

```text
7 element IDs
1 coefficient padding byte
7 integer element coefficients
1 signed formula charge
1 entry number
2 x u16 reference values
8-byte timestamp (OLE Automation date stored as f64)
2 unknown bytes
```

The formula supports at most seven elements. Compound chunks also contain seven `f64` real stoichiometric coefficients for fractional formulae.

## Why the Kaitai schema stays flat

Kaitai Struct is well suited to describing and visualising the physical records, including fixed-size bodies and ID-based switching. The ownership links are more naturally implemented in Rust because:

- CP and kappa records refer to phases by `phase_id_raw`.
- comments are stacked fragments rather than length-prefixed text;
- a compound group is terminated implicitly by the next chunk category;
- semantic validation requires cross-record state.

The `.ksy` therefore parses each 256-byte chunk independently and preserves unknown IDs as raw 255-byte bodies. This makes it safer for reverse engineering and large-file inspection.

## Required validation before Rust implementation

Test the schema against several databases and check that:

- the parsed chunk count equals `file_size / 256`;
- the first record is ID `9` and its magic is `CMPD`;
- every known body consumes exactly 255 bytes;
- compound groups follow the expected ordering;
- no common files contain unknown IDs;
- phase and range identifiers link consistently;
- fixed strings display correctly after trimming NUL and space padding.
