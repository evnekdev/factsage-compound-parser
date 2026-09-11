# Validation status

## Validated Windows facts

Windows is the primary validated platform because FactSage and the reference databases are Windows-based. The Rust implementation remains portable where practical.

The ignored `MS16BASE.CDB` fixture still validates as 755,712 bytes and 2,952 exact chunks. It round-trips byte-for-byte, and its raw bytes, streaming reader, index, diagnostics, and thermodynamic aggregate checks agree with the established expectations.

The read-only FactSage installation corpus validator scanned 13 `.CDB` candidates under the configured root:

| Metric | Result |
| --- | ---: |
| Candidates / total bytes | 13 / 15,345,664 |
| Valid Compound Databases | 12 |
| Valid chunks | 59,239 |
| Exact raw round trips | 12 / 12 |
| `from_bytes` = `from_reader` = `from_path` | 12 / 12 |
| Deterministic domain indexes | 12 / 12 |
| Domain failures / diagnostics | 0 / 0 |
| Unknown chunk IDs in valid databases | 0 |
| ID-11-bearing databases / records | 3 / 86 |
| ID-11 orphan / ambiguous links | 0 / 0 |

One candidate used a `.CDB` extension but had first ID 0 instead of the required Compound Database ID 9. It is cleanly rejected as `InvalidFirstChunkId`; its semantics are not inferred.

## Observed structural coverage

The valid corpus exercised every known ID: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, and 11. The aggregate raw histogram was:

```text
1: 10,714   2: 22,481   3: 394   4: 82   5: 1,156   6: 2
7: 13,589  8: 2,144  9: 12    10: 8,579 11: 86
```

Validated patterns absent from `MS16BASE.CDB` include CP IDs 3, 4, and 6; ID-11 records; compounds containing both CP and ID-11 records; stacked comments; multiple transition phases; and zero-width temperature ranges. The parser preserves and indexes all of them without diagnostics. Zero-width ranges are observed data, not automatically treated as malformed; CP evaluation retains its documented inclusive-bound policy.

All 86 ID-11 records had finite parsed numeric fields and valid temperature bounds. They linked to solid phases with observed raw IDs 101 through 108. Their equation, units, and coefficient meanings remain unverified.

## Robustness and I/O coverage

Synthetic tests cover every known chunk type, exact raw serialization, non-canonical floating-point bit patterns, unknown bodies, partial records, strict fixed-width edits, domain ordering, duplicate/orphan links, ID-11 ordering and locality, and thermodynamic typed errors.

Windows-compatible fault injection covers reader failure before input, mid-first record, at a chunk boundary, and mid-later record; writer failure after partial output; nonexistent and invalid paths; directory paths; Unicode and space-containing paths; and long paths where the local Windows configuration permits them. `ParseError` and `SerializeError` retain path context and underlying OS sources for path APIs.

Bounded `proptest` cases cover generated raw round trips, float bit patterns, unknown chunks, structural insertion/removal reversibility, setter locality, index rebuilding, and view traversal. Synthetic cargo-fuzz targets cover raw parsing, round trips, indexing, editor operations, and thermo access. Windows type-checks the targets; long sanitizer-backed execution remains a future robustness activity rather than a publication blocker.

## Inferred behavior retained conservatively

- Energy code 0 uses 4.184 conversion; code 1 is joule-based.
- Pressure code 0 is atmospheres and code 1 is bars; no conversion is provided.
- Phase state/index follows the existing Python threshold rules.
- Density exposes the Python-compatible floating remainder modulo 1,000,000; its unit and encoded high portion remain unknown.
- At adjacent shared CP endpoints, phase-level evaluation selects the lower-temperature interval. Individual raw range containment remains closed/inclusive.

## Unresolved semantics

- Physical meaning of unknown and reserved bytes;
- semantic distinction among CP IDs 2 through 6;
- magnetic, pressure-volume, transition, and ID-11 effective-G equations;
- density unit and high-order encoding;
- unusual phase-ID validity and negative field meaning;
- ID-11/kappa physical equation, units, and coefficient meanings;
- compound and phase writing semantics outside the controlled setters.

No proprietary records, text, formulae, coefficients, paths, or raw excerpts are committed or documented.

## FDB provider thermodynamic validation

Synthetic releasable tests validate the established 298.15 K integration rule
for powers -3, -2, -1, -0.5, 0, 0.5, 1, 2, and 3. They cover continuous and
independently discontinuous H/S range pairs, gaps, overlaps, reversed ranges,
mixed CP kinds, non-finite terms, missing CP, transition records, active and
inactive magnetic fields, pressure-volume blockers, and linked ID-11 blockers.
The optional installed-FDB audit remains aggregate-only and was not configured
for the latest provider-hardening run.
