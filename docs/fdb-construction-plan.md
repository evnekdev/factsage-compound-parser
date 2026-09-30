# FDB fresh-construction plan

## Purpose

Direct-modern authoring and Legacy -> Modern FactSage conversion both require
creation of a solution-side Function Database (FDB), with distinct naming and
zero-object policies.

The current crate is deliberately raw-authoritative and is strong at parsing,
validation, controlled edits, structural insertion/removal, and exact
serialization. It does **not** yet expose a provider-semantic API that can safely
construct a complete fresh FDB function family from thermodynamic intent.

Database Compare and the solution parser must not assemble 256-byte FDB/CDB
records themselves. This crate should own that construction boundary.

## Existing capability

The current implementation already provides the lower layers needed by a fresh
constructor:

- lossless parse/serialize for the shared CDB/FDB physical record family;
- typed raw chunk structures for database, compound, phase, CP, comments,
  kappa, and preserved unknown records;
- structural insertion/removal with deterministic index rebuild;
- domain grouping and reference validation;
- FDB ordinary-phase thermodynamic views;
- established energy-unit conversion and audited CP/H/S semantics;
- finite/range/continuity validation;
- strict serialization and reparse support.

Low-level `RawDatabase::insert_chunk` and `DatabaseEditor::insert_chunk` are
escape hatches, not an approved fresh-construction API. They intentionally allow
temporarily invalid ordering and require callers to know raw record layout.

## Required ownership boundary

Add a provider-owned construction layer:

```text
validated thermodynamic/function construction intent
        |
        v
FDB construction plan
        |
        v
provider builder
        |
        v
RawDatabase / RawChunk sequence
        |
        v
DomainIndex validation
        |
        v
serialize -> strict reparse -> semantic verify
```

The public caller supplies semantic intent and explicit provider policy, never
raw byte offsets, reserved fields, chunk padding, or physical record order.

## Minimum construction intent

The first builder may support a smaller direct-modern profile before the
Legacy translation profile, provided every required native byte is evidenced.
The two profiles must remain explicit.

A function construction request needs:

- database/header metadata required by FDB;
- database-global stoichiometric compound/formula identity;
- stable function object name, for example `PHAS_0000` or `PHAS_0000A`;
- phase kind and source/provider role;
- ordinary thermodynamic ranges or the established A/additional record form;
- H/S anchors, CP coefficients/powers, temperature bounds, and energy unit;
- explicit density/magnetic/pressure-volume disposition;
- any required source/status/comment metadata whose native encoding is known;
- provenance identity supplied by the caller for diagnostics/receipt binding.

Unknown or unevidenced required fields must produce a typed construction
blocker. They must never be filled by zero merely because the raw struct has a
numeric slot.

## Database-global stoichiometric grouping

The constructor must support multiple independently named function objects under
one stoichiometric compound/formula group.

Equal stoichiometry controls grouping only. Equal thermodynamic values must not
deduplicate, merge, or rename function objects.

This is required for FactSage solution FDBs where unrelated source phases can
contribute entries to the same formula group.

## Base/A pairing

The builder itself should not infer A companions from value equality. The
solution-provider conversion plan supplies explicit base/A identities.

Direct-modern functions are caller named and need no automatic A companion.
`FdbBuildPlan::new_fresh_modern` admits those base functions with empty Legacy
provenance. `FdbBuildPlan::new` preserves the strict translated
`<FILE phase ID>_<NNNN>[A]` naming and explicit one-to-one base/A pair.
Neither an omitted SLN formula qualifier nor an empty FDB compound-name field
removes the separate FDB composition group or formula label.

For rigorous Legacy -> Modern construction, callers may request both:

```text
FORMULA / PHAS_0000
FORMULA / PHAS_0000A
```

even when the A contribution is zero. If a target-version policy intentionally
omits a physically zero block, that must be an explicit serialization policy
above the raw builder, not an accidental consequence of dropping zero-valued
requests.

## Construction API shape

The exact Rust names are implementation details, but the public boundary should
look conceptually like:

```text
FdbBuildPlan
  database metadata
  ordered formula groups
    ordered function objects
      identity/name
      thermodynamic representation
      auxiliary contribution disposition
      source/provenance token

FdbBuilder::validate(plan)
FdbBuilder::build(plan) -> RawDatabase
```

Validation must be reusable without materialization so applications can preview
blockers before writing files.

## Validation rules

At minimum, preflight must reject:

- duplicate target function identity within one logical namespace;
- illegal or ambiguous compound/phase references;
- non-finite thermodynamic values;
- invalid or non-contiguous temperature ranges where continuity is required;
- coefficient/power shapes the native record family cannot encode;
- unknown required unit/reference conventions;
- unsupported active magnetic/pressure-volume/ID-11 contributions;
- construction that would silently lose a requested function object;
- ordering/grouping inconsistent with the domain grouping state machine.

After raw construction, always rebuild a `DomainIndex` and require a clean
validated view for the admitted profile.

## Milestone sequence

The FDB-C1 inventory is recorded in
[`fdb-construction-field-inventory.md`](fdb-construction-field-inventory.md).
FDB-C2 is represented by the sealed `fdb_build::FdbBuildPlan`: caller-provided
ordered groups and objects are validated without native records; typed errors
identify the object and field. Semantic validity and `native_blockers()` are
separate. The plan preserves explicit zero A objects and rejects active
auxiliary physics in the Legacy translation profile. The fresh-modern profile
uses caller-supplied names without Legacy source fields or an automatic A.
FDB-C3 is not implemented; the inventory lists its exact native evidence gaps.

FDB-C2 hardening replaced the floating ratio-bit key with exact rational
element ratios plus explicit semantic charge. Formula text remains a separate
label. A companions now reference typed base identities, and nonzero A plans
retain both phase and CP H/S fields. The target phase state is explicit and
must agree across a base/A pair; the provider still owns its native ID.
Provider blockers distinguish engineering, native evidence, scientific
semantics and verification with exact affected objects. The local paired
translation supports group-local solid ID and CP-link observations, but shows
that base/A role alone cannot select CP ID-2 versus ID-5. Every paired FDB
ID-2 has nonzero Cp and every ID-5 has zero Cp, including role exceptions.
The direct-modern edit sequence has one named solid function, no hidden A,
fresh ID-2 nonzero-Cp ranges, chained multi-range bounds and stream-order
entry numbers. It confirms fresh header/shared-field patterns without
supplying semantic charge/composition mapping, versioned unknown-byte and
timestamp policies, or independent acceptance of a provider-built file.
The [`FDB-C3 readiness review`](fdb-c3-readiness.md) therefore keeps native
materialization blocked. A conditional canonical ID-2 policy for fresh
zero-Cp ranges is described there; it is not a universal semantic equivalence.

### FDB-C1 — construction inventory

Map every field required to synthesize:

- database header;
- formula/compound group;
- ordinary phase/function object;
- ordinary CP ranges;
- A/additional function object.

For each field record one of:

- caller-provided semantic value;
- deterministic provider default with evidence;
- generated structural value;
- raw/native value not required by the admitted profile;
- unresolved blocker.

No field may be defaulted by omission.

### FDB-C2 — typed plan and blocker model

Implement a pure in-memory `FdbBuildPlan` plus typed validation errors.
No serialization yet.

Synthetic tests must prove:

- two equal-valued functions remain distinct;
- two phase families with equal stoichiometry share the intended formula group
  without sharing function identity;
- base and A objects remain one-to-one;
- zero A requests survive planning.

### FDB-C3 — raw materialization

Materialize the validated plan to `RawDatabase` using provider-owned raw record
construction. Reuse existing raw serialization; do not add a second writer.

Tests must cover deterministic ordering, generated IDs, chunk grouping, and
strict domain-index rebuild.

### FDB-C4 — thermodynamic verification

For every constructed ordinary function:

1. serialize;
2. reparse;
3. obtain the FDB thermodynamic view;
4. compare H/S/Cp/range semantics to the plan analytically.

Include multi-range functions and the exact legacy-to-FDB coefficient mapping
used by the solution provider.

### FDB-C5 — A/additional verification

Establish the exact native representation and verification profile for the
paired A object used by solution FDB conversion. Test both nonzero and explicit
zero construction.

If current provider evidence cannot prove a zero A physical encoding, retain
the requested identity in the construction plan and report a precise provider
blocker rather than silently omitting it.

### FDB-C6 — solution-provider integration

Expose the builder to `factsage-solution-parser`/Database Compare through a
provider-level API. The solution provider owns source G identity and matrix
coefficients; this crate owns FDB-native construction and validation.

The application receives only a sealed plan/result/blocker surface.

## Acceptance gate

Fresh FDB construction is not complete until all of the following hold:

- plan validation is independent of serialization;
- no raw-layout construction occurs outside this crate;
- serialize -> reparse succeeds;
- domain grouping/index validation succeeds;
- thermodynamic verification succeeds for the admitted subset;
- equal values never collapse structural identity;
- base/A identity is preserved exactly as requested;
- unsupported active auxiliary physics fails closed with a typed reason;
- tests exercise deterministic fresh construction from an empty/synthetic plan,
  not merely mutation of an existing FDB.
