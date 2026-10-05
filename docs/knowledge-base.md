# Native-format knowledge base

Status: CURRENT INDEX.

This repository owns the native semantics and transport of the shared FactSage CDB/FDB binary family. Database Compare consumes these facts through its provider adapter; it does not own CDB/FDB field meanings.

## Categories

1. Physical binary grammar — chunk IDs, 256-byte framing, fixed layouts, reserved bytes and round-trip preservation.
2. Native semantic structure — compound/phase/range grouping, phase IDs, references and native object relationships.
3. Thermodynamic native semantics — established H/S/Cp fields, units, OLE dates and bounded provider evaluation.
4. FDB construction/role evidence — CDB/FDB role assignment, ordinary Function-compatible profile, CP-kind evidence and native construction rules that are actually evidenced.
5. Physical/auxiliary fields — magnetic, pressure-volume, density and ID-11 structures, including unresolved semantics.
6. Editing/serialization — supported setters, locality and raw-authority rules.
7. Evidence/uncertainty — established, partial, unknown and warning-only native facts.

## Authority states

Use CONFIRMED, EVIDENCED, PARTIAL, HYPOTHESIS, UNKNOWN, WARNING_ONLY, OPAQUE_METADATA and SUPERSEDED. Unknown native meaning must not be promoted from a downstream application's assumptions.

## Canonical files

- `native-format-records.md` — structured native facts, unresolved meanings and CDB/FDB role evidence.
- `fdb-construction-evidence.md` — bounded native FDB construction observations migrated from downstream experiments.
- `format-overview.md` — physical chunk grammar.
- `domain-model.md` — native structural grouping and relationships.
- `semantic-rules.md` — established field decoding and units.
- `thermodynamic-semantics.md` — bounded thermodynamic semantics and provider evaluation.
- `serialization.md` — raw-authority and exact round-trip contract.
- `validation-status.md` — corpus/test evidence and unresolved semantics.

## Ownership boundary

- CDB/FDB native IDs, field meanings, phase-ID encodings, CP-kind meaning, density encoding, FDB-native construction rules and serialization behavior belong here.
- U2 objects, universal filtering and cross-provider conversion policy belong in `database-compare`.
- Solution DAT/SLN native semantics belong in `factsage-solution-parser`.
- Model equations not directly established as native CDB/FDB field semantics belong in `gem-rs`.

## Maintenance rule

Update the structured native record first. Detailed parser/thermodynamic documents may then be updated for grammar or evidence. Historical experiments remain evidence but must not override the current record.