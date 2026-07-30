# Validation status

## Validated facts

- The Kaitai layout and native parser both accept the ignored local fixture as 2,952 exact 256-byte chunks (755,712 bytes).
- The raw serializer reproduces the unmodified fixture byte-for-byte in memory. Reparsed serialized bytes retain the same chunk-ID sequence, grouped counts, and diagnostics.
- The streaming reader and byte-slice reader produce equal owned raw streams for the local fixture.
- The local fixture contains IDs 1, 2, 5, 7, 8, 9, and 10 only; no unknown chunks or kappa records occur there.
- Rust/Python aggregate parity agrees as recorded in [Python parity](python-parity.md).
- All fixture OLE values and density values accepted by their typed accessors are finite. All CP coefficients, powers, and bounds inspected by aggregate validation are finite and have non-reversed bounds.
- The fixture has 834 adjacent shared CP endpoints, zero strict interval-overlap phase sets, and zero gapped phase sets.
- The raw stream remains the only owned record store. `DomainIndex` contains chunk indexes, link vectors, and diagnostics only; semantic views borrow raw records instead of cloning them.

## Robustness coverage

Synthetic tests cover every known chunk type, exact raw serialization, non-canonical floating-point bit patterns, unknown bodies, reader partial records, strict fixed-width edits, domain ordering, duplicate/orphan links, and thermodynamic validation.

Bounded `proptest` cases cover generated valid raw round trips, arbitrary float bit patterns, unknown chunks, structural insertion/removal reversibility, setter byte locality, index rebuild correctness, and repeated view traversal. Synthetic cargo-fuzz targets cover raw parsing, raw round trips, indexing, editor operations, and thermodynamic access. Windows type-checks those targets; sanitizer-backed execution remains a Linux/WSL follow-up because the local MSVC toolchain lacks the required libFuzzer sanitizer runtime.

## Inferred behavior retained conservatively

- Energy code 0 uses the established 4.184 conversion; code 1 is joule-based. The fixture contains only those codes.
- Pressure code 0 is treated as atmospheres and code 1 as bars. No pressure conversion is provided.
- Phase state/index follows the existing Python threshold rules.
- Density exposes the Python-compatible floating remainder modulo 1,000,000. Its physical unit and encoded higher portion remain unknown.
- At an adjacent shared CP boundary, phase-level evaluation chooses the lower-temperature interval. Individual raw range containment remains closed/inclusive.

## Unresolved semantics

- Physical meaning of unknown and reserved bytes;
- semantic distinction among CP IDs 2 through 6;
- CP stored enthalpy/entropy anchor convention and integration rules;
- high-density encoding and density unit;
- phase-ID negative field meaning and unusual ID validity;
- kappa equation semantics;
- compound and phase writing semantics outside the controlled setters.

No proprietary CDB records, text, formulae, coefficients, or raw excerpts are committed or documented.