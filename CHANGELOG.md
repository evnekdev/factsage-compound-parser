# Changelog

All notable changes are documented here. The project follows an experimental 0.x API policy; public API changes are reviewed explicitly before release.

## 0.1.0 (release candidate)

- Added lossless native parsing, typed raw records, semantic grouping, borrowed indexed views, controlled edits, and byte-for-byte serialization for FactSage Compound Database streams.
- Added Windows-first release-candidate validation against an aggregate-only, read-only local FactSage corpus.
- Validated all known chunk IDs, including structural ID-11 preservation and exact raw phase-ID linking; no ID-11 physical equation or units are claimed.
- Added typed path-aware open/read/create/write errors while retaining underlying OS I/O sources.
- Added Windows-compatible injected I/O failure tests, local corpus validation, ID-11 regressions, bounded property tests, fuzz targets, and Windows fixture performance measurements.
- Confirmed package contents, MSRV 1.85.0, strict Rustdoc, dependency audit, and public CI coverage.

## Unreleased

No unreleased changes.