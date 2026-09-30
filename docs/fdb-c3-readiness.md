# FDB-C3 readiness after local paired-translation audit

**Decision: BLOCKED for native materialization.** The provider can implement
plan traversal, raw chunk assembly, serialization, strict reparse, domain
indexing and ordinary thermodynamic verification with existing lower layers.
It cannot yet assign every native byte for even one neutral, one-range function
without guessing header, shared-entry, compound, phase and CP fields. The
ignored local `data/v1` translation was examined before this decision. No raw
FDB construction is implemented by this review.

The field-level status checklist is
[`fdb-construction-field-inventory.md`](fdb-construction-field-inventory.md).
`FdbBuildPlan::native_blockers()` emits field-specific records with a
provider-neutral class, exact object, known fact, missing rule, and smallest
evidence artifact. It includes an engineering blocker for the unimplemented
builder and a verification blocker for independent FactSage acceptance. Caller
H/S/Cp values, evidenced chunk IDs, native units and inactive physical tails
are not misclassified as new evidence needs.

## Evidence precedence

Before requesting any new manual FactSage experiment, inspect the existing
local-only paired Legacy -> Modern translation corpus recorded in
`database-compare/architecture/local-v1-translation-evidence.md` and stored
under ignored `database-compare/data/v1/`.

This local corpus is the primary evidence source for current FDB-C3 blockers.
The intended order is:

```text
provider code and tracked contracts
    +
local paired data/v1 translation evidence
    +
historical dbsolution evidence where useful
        ↓
resolve or narrow blockers
        ↓
only then request the smallest new controlled FactSage experiment
```

The corpus itself remains confidential and untracked. Only generalized,
redacted structural conclusions may enter tracked code, tests, or
documentation. If `data/v1/` already answers an E1/E2/E3 question, that
experiment is not required.

## Evidence audit

- **Provider code:** `RawCommonHeader.charge_raw` is signed `i8` and occurs on
  ID-1, ID-7 and CP bodies. `CompoundView` exposes the raw ID-1 header but has
  no charge-specific semantic accessor. `DomainIndex` links CP by exact phase ID
  within a compound. The parser/serializer preserve unknown and padding bytes;
  that behavior proves no fresh default.
- **Tracked solution contract:** source charge is modeled separately from
  formula text (`i32` in solution models). Earlier paired examples establish
  base/A naming and one exceptional A entropy sign, but do not establish a
  universal CP kind or fresh native header initialization.
- **Local paired Legacy/Modern corpus:** receipt-admitted source G entries were
  matched by source identity and encounter index to FDB names and SLN external
  references. FILE phase ID, zero-based suffix and A naming held for matched
  entries. Matching matrix coefficients transferred to endmember references
  without conversion or value-based merging. Multiple model families, charged
  groups, base/A cases and zero objects occur. The FDB also has functions not
  represented by parsed source G entries. Both CP ID-2 and ID-5 occur under
  base and A names; role does not determine chunk kind. Zero objects can be
  emitted or omitted. The corpus has no neutral FDB group.
- **Native field variation:** solid phase IDs increase within each group and
  repeat across groups; negative IDs are their arithmetic negation in the FDB
  and comparison CDBs. CP links are group-local. Shared composition and raw
  charge repeat within groups; entry numbers and timestamps vary. Strict
  simple integer FDB labels match atomic-number IDs and integer/real
  coefficients, but comparison CDBs do not follow the same pattern. Repeated
  zero padding and header fields are observations, not fresh defaults.
- **Historical Python:** `dbsolution` reads Legacy charge and derives modern
  endmember charge with `chemformula`. No FDB byte constructor, fresh phase-ID
  allocator, entry/reference counter rule, or shared-header padding rule was
  found there. These hints do not override the current Rust provider contract.
- **API boundary:** The rigorous plan carries exact rational element ratios,
  explicit semantic charge, source G identity, deterministic target name,
  target phase state and a typed base reference. It carries explicit phase H/S
  for nonzero base and A objects. Target names are unique within a full
  formula-group namespace;
  source G roles remain globally distinct. It contains no raw offsets or byte
  padding fields.

## Remaining experiments after the local audit

All inspected native files must remain local and uncommitted. Record the
FactSage version and creation action; compare only aggregate field decisions in
project documentation.

### E1 — minimal neutral fresh FDB

**Question:** Which non-scientific fields are fixed, generated, copied, or
versioned in a freshly created ID-9/ID-1/ID-7/CP stream?

**Input and action:** In FactSage, create one fresh Function Database with one
neutral formula group, one ordinary function named from a known phase ID, one
Cp interval, known H/S/Cp, known units and explicit date/comment if the
UI allows them. Create a second file under the same version with one added
ordinary function in the same group; retain creation order and timestamps.

**Files and fields to compare:** The two generated FDBs and their recorded
semantic inputs. Inspect ID-9 padding/unknown/read flag/date; every shared
header's element IDs, integer coefficients, charge, entry/reference values,
timestamp, padding; ID-1 compound/formula names, real coefficients, reserved
bytes; ID-7 positive/negative phase IDs and padding; CP kind, unknown/padding and
phase link. The second file separates generated counters from constants.

**Decision enabled:** Write explicit field rules and deterministic ID/link
allocation for the minimal neutral subset and test the paired solid allocation
against fresh creation. If any bytes vary without an identified input or
version rule, keep that field blocked.

### E2 — charge and exact-ratio variation

**Question:** Does semantic charge map directly to `charge_raw` and repeat on
ID-1/ID-7/CP? How do integer and real native coefficients encode scale?

**Input and action:** Under the same FactSage version, create otherwise equal
neutral and +1 charged groups with the same Fe/O ratio, then a second group
whose written coefficients are proportional (for example 3:4 versus 6:8) and
one fractional-ratio group if the UI admits it. Use the smallest possible
single-function definition in each group.

**Files and fields to compare:** Paired FDBs and source composition/charge
entries. Inspect `charge_raw`, `element_ids[7]`, integer coefficients,
`real_stoichiometric_coefficients[7]`, compound/formula labels, and repetition
on phase/CP shared headers.

**Decision enabled:** Confirm or reject direct charge mapping, neutral zero,
record repetition, element ordering, and ratio/scale encoding. Decide the
admitted native charge range separately from the wider semantic `i32` type.

### E3 — base/A and zero-object native form

**Question:** Which zero objects are physically representable or omitted under
a named FactSage version, and what selects ID-2 versus ID-5 for base/A Cp?

**Input and action:** Use a controlled Legacy source with two G entries: one
nonzero base/A pair and one zero base/A pair. Import to Modern with FactSage;
retain the source Legacy text, resulting FDB and SLN, version and importer
settings. Include a deliberate A entropy sign probe if possible.

**Files and fields to compare:** Source G leading pairs and counted intervals,
generated ID-7/ID-2/ID-5 records, A phase H/S, CP bounds/powers, zero-object
presence, and external SLN references. Compare source and target structurally
and numerically without printing proprietary record contents in logs.

**Decision enabled:** Establish the model/function-specific CP kind condition
and versioned NativeFactSage zero-object policy separately from rigorous
semantic identity. Resolve the exceptional A entropy sign rule before the
solution adapter claims scientific equivalence.

## Safe engineering work now

The existing parser already knows the chunk grammar, exact ID-2/ID-5 body
shape, raw serialization, strict reparse, grouping and ordinary H/S/Cp
validation. The semantic plan now records target phase state and rejects a
base/A pair with mismatched states. A builder skeleton, plan-order traversal,
typed generated-ID map and post-build verification pipeline could be
implemented without writing native chunks. Actual byte materialization waits
for E1 and a CP kind rule; charged support waits for E2; zero base/A and Legacy
A conversion wait for E3. This review leaves those engineering tasks
unimplemented because no complete bounded native profile is yet evidenced.
