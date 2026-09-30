# FDB-C3 readiness after joint fresh-modern and translation audit

**Decision: BLOCKED for native materialization.** The provider can implement
plan traversal, raw chunk assembly, serialization, strict reparse, domain
indexing and ordinary thermodynamic verification with existing lower layers.
It cannot yet assign every native byte for a bounded fresh, one-function FDB
without guessing header, semantic charge/composition, timestamp and some
record-kind fields. The ignored `data/v0` direct-modern snapshots and `data/v1`
official Legacy translation were examined separately. No raw FDB construction
is implemented by this review. FDB-C1 and FDB-C2 remain GO; FDB-C3 remains
BLOCKED.

The field-level status checklist is
[`fdb-construction-field-inventory.md`](fdb-construction-field-inventory.md).
`FdbBuildPlan::native_blockers()` emits field-specific records with a
provider-neutral class, exact object, known fact, missing rule, and smallest
evidence artifact. It includes an engineering blocker for the unimplemented
builder and a verification blocker for independent FactSage acceptance. Caller
H/S/Cp values, evidenced chunk IDs, native units and inactive physical tails
are not misclassified as new evidence needs.

## Evidence precedence

The directly authored modern snapshots under ignored `database-compare/data/v0/`
are the primary evidence for fresh writing. The paired translation under ignored
`database-compare/data/v1/` is primary for Legacy conversion behavior. Provider
code and tracked contracts constrain both. The intended order is:

```text
provider code and tracked contracts
    +
direct-modern data/v0 construction evidence
    +
paired data/v1 Legacy translation evidence
    +
historical dbsolution evidence where useful
        ↓
resolve or narrow blockers
        ↓
only then request the smallest new controlled FactSage experiment
```

Both corpora remain confidential and untracked. Only generalized,
redacted structural conclusions may enter tracked code, tests, or
documentation. The local receipts alone select included native members; ignored
members were not read.

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
  charge repeat within groups. FDB entry numbers increment once per group
  record in stream order; the comparison CDBs do not all follow that rule.
  CP timestamps copy the ID-1 timestamp in all local native members, while
  phase timestamps can differ. Strict simple integer FDB labels match
  atomic-number IDs and integer/real
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

## Direct-modern fresh construction audit

The receipt-admitted sequence contains a header-only database, an empty named
function, the same function with one nonzero-Cp range, a reference/density edit,
and a three-range edit. These are direct-modern snapshots, not Legacy imports.
The files round-trip through the provider parser and serializer byte-for-byte.
The observations below are bounded to this one controlled edit sequence; the
FactSage version, original UI inputs, and independent creation history were
not recorded in the receipt.

| Object | Fresh-modern observation | Remaining rule |
| --- | --- | --- |
| ID-9 | One header exists even in the empty file. Read flag and all three padding regions are zero; the two unknown regions are stable and contain nonzero bytes, while date/comment remain unchanged through edits. The first unknown region agrees with the translated FDB; the second differs. | Version-specific initialization of unknowns, read flag and date/comment policy; equality across edits does not prove a universal constant. |
| Shared headers | Element identifiers, integer coefficients, charge and coefficient padding repeat from ID-1 through ID-7 and CP. Per-kind unknown bytes are stable and agree with translated counterparts. Entry numbers advance with record order from a common group start, including added ranges. | Input-linked composition/charge mapping, versioned unknowns, larger-group counter limits and fresh writer acceptance. |
| Names and composition | The fresh ID-1 compound-name field is empty while its formula label is populated. Real coefficients, reserved strings, units and composition slots remain stable through these edits. The phase name remains a nonempty caller-style name. | Formula/compound alias policy and input-linked element order, scale, charge and unit assignment. A blank SLN formula qualifier never means absent FDB group metadata. |
| Phase | Each nonempty snapshot has one solid-region ID-7. The ID stays fixed as ranges are added, its negative field is its arithmetic negative, and all CP ranges link to that same ID. The empty function has ID-7 but no CP. No hidden A companion appears. | Allocation for a second fresh function and non-solid states. Fresh empty-function form is observed; translated zero-object omission remains separate. |
| Reference and density edit | The phase density and phase shared-reference fields change. The group and CP reference fields remain zero; the CP body otherwise remains unchanged apart from its timestamp. Other physical-tail fields remain the same. | Mapping of a user reference to the two native slots; no-reference and inactive-density policies can be bounded, but active metadata cannot yet be generated. |
| Timestamps | Fresh ID-1, ID-7 and CP timestamps agree within each snapshot and change with edits. The ID-9 database date does not change with those entry timestamps. | Clock source, rounding, creation-versus-save semantics and deterministic policy. Translation CP copies ID-1, but translated ID-7 can differ. |
| CP | Every fresh nonzero-Cp range uses ID-2. The three-range file stores separate CP records under one phase, with adjacent bounds chained, local entry progression, copied group timestamp, and distinct range H/S anchors. The last coefficient and power slots are zero in this sequence, but a zero coefficient does not imply a zero power in every slot. | A fresh zero-Cp kind selector; source-input mapping for the eight stored Cp slots and unused-power policy. |

No fresh semantic neutral/charged pair is declared. The observed raw charge
cannot be interpreted as semantic charge from formula spelling. No controlled
ratio change or second function was admitted. The user-observed UI behavior
around empty/zero Cp and later ranges is useful for experiment design, but it
does not establish a binary rule.

## Exhaustive CP-kind falsification pass

The local-only map includes every receipt-admitted FDB CP record, matched
source G entries and direct QKTO functions. It tests all represented model
families, both base/A names, source ordinary-range counts, direct source-range
matches, exact Cp coefficient zero status, phase links, and emitted/omitted
cases. It does not infer source identity by numerical equality.

**Observed invariant:** every paired FDB ID-2 CP record has at least one
exactly nonzero Cp coefficient; every ID-5 record has all-zero Cp coefficients.
Both kinds occur under base and A names. Every identity-and-order-linked
counted base range with nonzero source Cp uses ID-2, and every exact-zero
source Cp range uses ID-5 across the represented model families.
The same zero/nonzero split for IDs 2/5 appears in both comparison CDBs, but
their additional zero-Cp ID-3 records show that zero Cp alone does not select
a kind across all native databases. No counterexample to the restricted FDB
invariant was found. A fresh FDB with controlled zero/nonzero Cp remains
necessary to show whether this invariant is a constructor rule. Added and
generated source contributions need a separate, lossless Cp reduction.

The full local translation has only solid FDB phases. Its phase IDs are
group-local and consecutive, and their negative fields are arithmetic
negations. CP links point within the group. Entry numbers are consecutive in
ID-1/ID-7/CP stream order from a uniform group start. Both reference slots
are zero in the FDB, but comparison CDBs include nonzero references. These
relationships narrow fresh-format tests without proving version defaults or
non-solid allocation.

The emitted ID-5 A objects in this translation retain the source leading H
anchor and reverse the sign of nonzero source S. Earlier small paired examples
mostly retained the S sign. ID-2 A objects do not follow that direct leading
anchor mapping. No universal A sign condition follows from the combined
evidence. Emitted objects are referenced by Modern SLN and omitted objects
are not, but tested source-side zero, matrix, model and range features still
leave ambiguous emission cases. A NativeFactSage zero-object policy remains
blocked.

For identity-linked base functions, counted source-range cardinality and
upper-temperature bounds transfer in encounter order across represented
models. Direct H/S/Cp numeric conversion holds for a subset, while other
base ranges have transformed anchors or terms. The earlier small-fixture
conversion table therefore remains a bounded observation, not a universal
Legacy-to-FDB transform. The native builder can accept explicit target H/S/Cp
without solving the later source-to-plan adapter.

## CP-kind decision and fresh-write policy

The translation corpus has a reproducible output split: every observed ID-2
range has nonzero Cp, every ID-5 range has exactly zero Cp, and every
identity-linked counted base source range follows that split. The direct-modern
corpus exercises nonzero Cp only, using ID-2 in both single- and multi-range
functions. These observations support ID-2 for the admitted fresh nonzero-Cp
profile. They do not prove that Cp zero status causes the native kind choice in
a fresh writer. No observed native or provider thermodynamic interpretation
establishes a broader semantic difference between the two same-shaped bodies.

The candidate serialization policy is: preserve the original kind when editing
an existing file; use ID-2 for fresh nonzero-Cp ranges; consider canonical ID-2
for fresh zero-Cp ranges only after strict provider reparse/domain/thermodynamic
verification, independent FactSage acceptance, and load/open/save stability of
values and references. This is a policy candidate, not a claim that ID-2 and
ID-5 are universally interchangeable. Fresh zero-Cp construction remains a
policy/evidence blocker until those checks pass.

## Remaining FDB-C3 gates

| Object or capability | Class | Exact missing rule or action |
| --- | --- | --- |
| ID-9 and per-kind uninterpreted bytes | EVIDENCE_REQUIRED | Record the FactSage version and fresh initialization policy; observed stable bytes cannot be copied into a general constructor without provenance and independent acceptance. |
| Shared composition and charge | EVIDENCE_REQUIRED | Pair recorded semantic neutral and charged fresh inputs with their native element IDs, integer/real coefficients and signed charge. Neither corpus supplies that correspondence for a fresh neutral group. |
| Formula and compound names | EVIDENCE_REQUIRED | Define fresh label and absent/present compound-name policy separately from optional SLN formula qualification. Translation aliases are a different profile. |
| Timestamp assignment | POLICY_REQUIRED | Choose and validate a versioned fresh timestamp source/rounding rule. The ID-9 date remains stable while entry timestamps change; translation follows another phase-timestamp pattern. |
| More than one fresh function and non-solid states | EVIDENCE_REQUIRED | Confirm fresh phase-ID allocation and links beyond the observed one solid function. A one-solid-function profile can exclude these. |
| Active references and density | EVIDENCE_REQUIRED | Map user reference inputs to phase reference slots and density units. A bounded no-reference, inactive-density profile can exclude these. |
| Fresh zero-Cp kind and shorter Cp term lists | POLICY_REQUIRED | Either prove a fresh selector or validate canonical ID-2; establish unused coefficient/power filling. A nonzero, explicit eight-slot profile can exclude these. |
| Legacy A conversion and zero-object omission | SCIENTIFIC_SEMANTICS_REQUIRED / POLICY_REQUIRED | Retain rigorous identities; resolve source-to-target A reduction/sign and versioned omission separately. Fresh functions have no automatic A companion. |
| Provider materializer and verification pipeline | ENGINEERING_ONLY | Implement only after every byte for an admitted profile is supported; then serialize, strict-reparse, index and verify thermodynamics. |
| Provider-built file acceptance | EVIDENCE_REQUIRED | Open the constructed FDB in FactSage and verify reference/value stability through load/open/save. Self-reparse is insufficient. |

No nontrivial profile has every required native byte and input mapping. In
particular, restricting to one solid function, nonzero Cp, no active references
or density, and explicit H/S/Cp still leaves charge/composition, timestamps,
versioned uninterpreted bytes and fresh label policy unresolved. The
materializer is therefore not implemented. The semantic plan now explicitly
separates fresh caller names from Legacy-derived names without weakening the
Legacy pairing contract.

## Smallest next controlled experiment

Create two otherwise identical, directly authored modern one-function FDBs in
one recorded FactSage version: one explicitly neutral and one explicitly +1
charged, with the same simple integer composition, one nonzero-Cp interval and
no active reference or density. Keep a local receipt declaring each included
FDB and record the exact UI composition, charge, formula/function labels,
creation action, save action and times. The files remain ignored under `data/`.
Compare only categorical field behavior in tracked results. This pair can
establish the first input-linked fresh charge/composition rule, test whether
header and per-kind unknown bytes are version constants, and narrow the
entry-timestamp source. It does not by itself authorize a writer or settle
fresh zero-Cp policy.
