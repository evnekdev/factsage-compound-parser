# Changelog

All notable changes are documented here. The project follows an experimental 0.x API policy: `0.1.x` may evolve, breaking changes are documented explicitly, and patch releases should not silently introduce avoidable breaking changes.

## Unreleased

No unreleased changes.

## 0.1.0 - 2026-07-30

This is an experimental release. It is not a production-readiness claim or official FactSage compatibility certification.

- Added lossless native parsing for the fixed-width FactSage Compound Database format, including all known chunk IDs, preservation of unknown chunks, reserved fields, padding, and original floating-point bits.
- Added indexed borrowed semantic views over one authoritative raw database, including compound and phase traversal, diagnostics, comments, phase-ID linking, and structural raw insertion/removal.
- Added controlled editing for established fields and streaming input/output while preserving unrelated bytes and records.
- Added typed unit accessors, OLE Automation dates, density decoding, and stored heat-capacity expression evaluation; integrated thermodynamic calculations remain unsupported.
- Added structural-only ID-11 handling with exact raw preservation and phase-ID linking. Its physical equation, units, and coefficient meanings remain unverified.
- Validated Windows-first behavior against 12 valid local Compound Databases with exact in-memory round trips; the corpus scanner is read-only, ignored, and never distributed.
- Added Windows stable and Rust 1.85.0 MSRV CI, secondary portability checks, dependency auditing, property tests, fuzz targets, strict Rustdoc, packaging checks, and I/O fault coverage.

Known limitations include reverse-engineered unknown fields, CP integration conventions, density units, ID-11 semantics, and the experimental 0.x API stability policy.