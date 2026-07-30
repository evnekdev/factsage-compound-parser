# Validation status

## Validated facts

- The Kaitai layout and native parser both accept the ignored local fixture as 2,952 exact 256-byte chunks (755,712 bytes).
- The raw serializer reproduces the unmodified fixture byte-for-byte in memory, and the serialized bytes parse and group to the same aggregate counts and diagnostics.
- The local fixture contains IDs 1, 2, 5, 7, 8, 9, and 10 only; no unknown chunks or kappa records occur there.
- Rust/Python aggregate parity currently agrees as recorded in [python parity](python-parity.md).
- All fixture OLE values are finite and in the accepted Automation Date range. All fixture phase density values are finite. All CP coefficients, powers, and bounds are finite, with no reversed intervals.
- The fixture has 834 adjacent shared CP endpoints, zero strict interval-overlap phase sets, and zero gapped phase sets.

## Inferred behaviour retained conservatively

- Energy code 0 uses the established 4.184 conversion; code 1 is joule-based. The fixture contains only those codes.
- Pressure code 0 is treated as atmospheres and code 1 as bars. No conversion is provided.
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
## Release-foundation checks

The repository now declares `rust-version = "1.85.0"` and passes `cargo +1.85.0 check --all-targets --all-features` plus `cargo +1.85.0 test --lib --all-features`. Stable checks cover formatting, Clippy, all-target tests, documentation tests, example compilation, warnings-as-errors rustdoc, and packaging. GitHub Actions extends this to Linux, Windows, and macOS.

Dependabot monitors Cargo and GitHub Actions. The security workflow runs `cargo audit`; pull requests receive dependency review where GitHub provides it. These checks do not replace malformed-input robustness testing.