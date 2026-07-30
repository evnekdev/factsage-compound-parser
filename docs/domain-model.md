# Semantic domain model

## Ownership and views

`RawDatabase` is the only owned physical record store. `DomainIndex::build` validates and records relationships as physical chunk indexes; it never clones compound, phase, range, comment, unknown, padding, reserved, or ID-11 raw data. `DatabaseView<'a>` borrows both the raw stream and its matching index. `CompoundView`, `PhaseView`, and range views are cheap read-only projections over those indexed raw chunks.

`Database` owns one `RawDatabase` plus its matching `DomainIndex`. `DatabaseEditor` owns the same raw-authoritative arrangement lazily: structural edits discard its cached index and `view()` rebuilds it when needed. Use `RawDatabase::chunks()` for raw inspection and `Database::view()` or `DatabaseEditor::view()` for semantic traversal.

## Grouping state machine

The index consumes the flat stream in this order:

```text
database header
compound
phase*
(CP | ID-11)*
comment*
compound ... or end of file
```

The first chunk must be the ID-9 `CMPD` database header. A compound may have no phases, ranges, or comments. Once a range has occurred, a later phase is a fatal ordering error. Once a comment has occurred, a later phase or range is a fatal ordering error. `DomainIndex::build` returns `DomainError` for these known-ordering failures and does not return a partial index.

Unknown IDs inside an active compound group remain in the authoritative raw stream, are indexed as unknown chunks, and yield an `UnknownChunk` diagnostic. An unknown ID where a compound is required is a fatal ordering error.

## Phase IDs and labels

State decoding follows the established Python behavior:

| Raw ID condition | State | Index |
| --- | --- | --- |
| greater than 990 | aqueous | raw ID minus 990 |
| greater than 900 | gas | raw ID minus 900 |
| greater than 800 | liquid | raw ID minus 800 |
| otherwise | solid | raw ID minus 100 |

The compact label uses uppercase state prefixes: `S1`, `L2`, `G2`, or `AQ1`. The ChemApp-style label uses lowercase prefixes and omits the suffix for index 1: `s`, `l2`, `g2`, or `aq`.

Zero and negative calculated indexes are preserved instead of rejected and produce a `SuspiciousPhaseId` diagnostic.

## Range linking and ID-11 preservation

After phases have been collected for a compound, CP and ID-11 records link by exact equality of `phase_id_raw` within that compound.

- CP IDs 2 through 6 remain distinct through `HeatCapacityKind`.
- ID-11 records become `PhysicalPropertyRangeView` values in physical stream order.
- A single matching phase receives the range.
- A missing phase leaves the range indexed as an orphan with a typed diagnostic.
- Multiple matching phases leave the range indexed as an ambiguous orphan with a typed diagnostic.
- Non-finite or reversed temperature bounds are diagnostics only; original raw fields remain available.

Windows corpus validation exercised 86 ID-11 records across three databases. All linked unambiguously, remained in stream order, and round-tripped byte-for-byte. This validates structural offsets, signed raw phase-ID linking, and preservation only. It does not validate an equation, units, or coefficient semantics.

`CompoundView::phases`, `PhaseView::heat_capacity_ranges`, `PhaseView::physical_property_ranges`, `CompoundView::orphan_ranges`, and `CompoundView::unknown_chunks` traverse relevant raw records without allocating or copying them.

## Comments, text, and diagnostics

Comment chunks are consecutive fixed-width fragments. `CompoundView::joined_comment` decodes each fragment as Windows-1252, trims storage padding only, and joins fragments without introducing separators.

Compound names, formulae, and phase names are fixed-width ASCII-compatible fields. View accessors return `Result<Cow<str>, TextDecodeError>` so invalid bytes remain explicit. Their raw fixed arrays remain available through raw accessors.

`DatabaseView::diagnostics` returns non-fatal typed issues in physical stream order. Each diagnostic carries a zero-based source chunk index, absolute byte offset, and compound index when available. Diagnostics never mutate, reorder, or affect serialization.