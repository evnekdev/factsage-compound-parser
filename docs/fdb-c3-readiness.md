# FDB-C3 readiness after joint fresh-modern and translation audit

**Decision: BOUNDED-GO for FreshModern construction and bounded Legacy zero-A construction, with multiple native acceptance probes.** FDB-C1 and FDB-C2
remain GO. The fresh-modern native writer uses the bounded
construction policy in
[`fdb-bounded-fresh-profile.md`](fdb-bounded-fresh-profile.md). No remaining
scientific/domain-policy question requires human input for that profile.
`FdbBuildPlan::materialize_fresh` now assembles raw chunks from caller-supplied
controlled templates, serializes and strict-reparses, builds a `DomainIndex`,
and checks Cp/H/S/G at the endpoints and midpoint of each range. A synthetic
probe generated from receipt-admitted local templates passed these internal
checks. FactSage 7.3 rejected the first generated FDB when opened with a native
empty-SLN companion. A native one-function control opened and displayed its
function with the same companion. Controlled local mutations then showed that
replacing only the native control's ID-9 date with the generated date makes
FactSage reject the file, while replacing only its entry timestamps leaves
the function visible. Replacing only composition/group fields makes the file
open without showing a function. A further control that retained the complete
native ID-9 header and composition but replaced thermo and function name with
generated values opened and showed the function. A generated-composition probe
with the native header opened without showing its function when hydrogen occupied
the first element slot. Moving hydrogen to the last slot made that same
synthetic composition and formula label visible in FactSage. The revised Rust
writer then produced the same synthetic group. FactSage opened it, showed the
Function, saved it and reopened it successfully. The FDB and SLN remained
byte-identical to local pre-save copies. This establishes one controlled
open/save acceptance, while additional shapes retain a verification blocker.

The materializer now also compares reparsed formula, charge, element IDs and
coefficients, function names and IDs, phase anchors, and CP identity/bounds
against the plan. Synthetic multi-function/charged-group and hydrogen-only
probes pass this stricter internal check. Both generated files opened visibly
in FactSage 7.3, including two neutral Functions and one charged Function in
the multi-group probe. A separate generated LegacyTranslation file with a
physically omitted zero A also opened and displayed its base Function. A second
LegacyTranslation probe used an ID-5 exact-zero Cp range with its native power
slots copied from a version-compatible translated example; FactSage opened it
and displayed the Function. A synthetic QKTO SLN paired with the generated FDB
then opened with its solution and endmember visible; its single active `CpXp`
term resolved uniquely to the generated Function in the bundle resolver. A
wrong-FDB control failed that resolution. Native thermodynamic evaluation of
the referenced solution remains a separate check.

The bounded profile resolves the previously open fresh-writer choices by policy:
copy opaque ID-9/default bytes from the controlled empty-FDB profile, use OLE
Automation dates, parse formulas with `chemformula`, encode periodic-table
atomic numbers as one-byte element IDs, use formula encounter order among
non-hydrogen elements and place hydrogen last in native slots, subject to
automatic corpus confirmation, use integral formula coefficients with matching
`f64` real coefficients, and retain the established bounded charge offset.
Advanced real-stoichiometry, active nonzero references, advanced volumetric
families and Legacy source-to-target H/S/Cp/A reduction remain outside the first
profile rather than blocking it. Fresh zero-Cp is admitted as ID-2. Semantic
aggregate state does not select FDB Function ID bands: FDB Functions use the
group-local pseudo-solid 101+ series.

FDB-C3 is BOUNDED-GO for the implemented FreshModern profile and the admitted
Legacy zero-A subset. The first native failure was isolated, corrected, and
followed by successful provider-built acceptance. Hydrogen-only,
multiple/charged, translated zero-A ID-2/ID-5, and paired SLN/FDB reference
probes also opened visibly. This does not establish native acceptance for every
combination of groups, charges, ranges, or active-A forms.

The field-level status checklist is
[`fdb-construction-field-inventory.md`](fdb-construction-field-inventory.md).
`FdbBuildPlan::native_blockers()` emits field-specific records with a
provider-neutral class, exact object, known fact, missing rule, and smallest
evidence artifact. FreshModern now has only broader verification/hardening work. LegacyTranslation
reports an engineering blocker only when a plan falls outside the bounded
zero-A subset, such as active A, zero-base, empty-range, or mixed-kind cases. Caller
H/S/Cp values, evidenced chunk IDs, native units and inactive physical tails
are not misclassified as new evidence needs.

## Human-in-the-loop assessment

For the bounded FreshModern and bounded Legacy zero-A profiles:

```text
Additional user input required now: NO
Additional domain-policy decisions required now: NO
Additional controlled user-created evidence required now: NO
Remaining work: verification hardening plus out-of-profile extensions (not bounded-GO blockers), including native SLN thermodynamic evaluation and active-A reduction
```

The reusable `fdb_native_rule_audit` executable tested every FDB fixture
admitted by the local fresh and translated receipts. All eligible fresh records
matched the charge, element-order, integer/real coefficient, shared-header,
entry, phase-ID and CP-link rules. Translated records matched the structural
rules and every ordinary formula case eligible for this profile. Some
translated labels or compositions fall outside ordinary `chemformula` parsing;
the audit records those cases separately in ignored local results and does not
promote them as fresh-writer examples. No native-rule contradiction was found.
The receipt, detailed counts and fixture identities remain local.

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
  base/A naming; their only nonzero ID-5 A source entropy supports the same
  negation as the larger translated evidence. They do not establish fresh
  native header initialization.
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
| ID-9 | One header exists even in the empty file. Read flag and padding are stable; unknown regions are opaque. Isolated date and comment edits caused FactSage 7.3 rejection. | Bounded writer copies the complete controlled empty-FDB ID-9 template unchanged and requires matching plan metadata. No semantic interpretation of opaque bytes is required. |
| Shared headers | Element identifiers, coefficients, charge and per-kind opaque bytes repeat consistently within groups; entry numbers advance in physical record order from a common group start. | Bounded writer uses `chemformula`, non-hydrogen encounter order followed by hydrogen, atomic-number IDs, native charge `semantic + 50`, template-derived opaque bytes, and rejects before entry-number rollover. |
| Names and composition | Fresh ID-1 compound name is empty while the formula label is populated; simple integer cases align integer and real stoichiometry. | Preserve exact validated caller formula spelling; move hydrogen to the last native slot while keeping other elements in encounter order; write integral and matching `f64` coefficients in that slot order; fresh `compound_name` defaults empty. |
| Phase | Observed fresh Functions occupy the 100-series, negatives are arithmetic negations, and CP links use the exact positive Function ID. Empty fresh Function is ID-7 without CP. | FDB construction uses group-local pseudo-solid IDs `101, 102, 103, ...` in Function encounter order regardless of semantic aggregate state. |
| Reference and density edit | Phase density is active in ordinary files; reference slots can be zero. | References default to zero. Ordinary density uses the low/remainder density portion with no advanced family code; advanced volumetric families and active references are extensions. |
| Timestamps | Fresh ID-1, ID-7 and CP timestamps agree within each changed group; ID-9 date is independent. | OLE Automation dates. New/materially changed groups use current save/build time consistently; unchanged groups may preserve timestamps. |
| CP | Fresh observed nonzero-Cp ranges use ID-2; multi-range bounds are chained and retain source order. | Fresh construction uses ID-2 including zero-Cp. Preserve supplied coefficient/power pairs; zero-fill only unused coefficient and unused power slots. |

The original edit sequence had no declared neutral/charged pair or controlled
ratio change. The later charged-function probe below adds a user-declared
neutral/charged contrast. The user-observed UI behavior around empty/zero Cp
and later ranges is useful for experiment design, but it does not establish a
binary rule.

## Added fresh charged-function probe

The new receipt-admitted direct-modern snapshot was compared against the
earlier single-function snapshot and internally as a neutral/charged pair.
The declared original function was matched uniquely by formula group plus
phase name. Phase name alone matches both groups and cannot identify it.
Provider parsing/serialization and an independent byte-offset probe agree on
the categorical differences. No per-fixture names or coefficients are retained
in this comparison.

| Comparison | Supported conclusion | Limit |
| --- | --- | --- |
| Original neutral function across snapshots | ID-9 is unchanged. ID-1 and ID-7 change only their timestamps. The original CP changes timestamp and upper temperature bound; its other fields remain unchanged. | The second snapshot is not a pure add-only delta because the original range bound also changed. Do not attribute that bound change to charge. |
| Neutral vs charged within the new snapshot | Two ID-1 groups have the same element IDs, integer and real coefficients, units, reserved/compound-name fields, phase name, phase H/S and CP data. Only raw charge, formula label and per-group timestamp differ. | The user declares neutral versus charged, but the exact entered signed charge magnitude, formula-input semantics and FactSage version are not recorded. |
| Charge representation | `charge_raw` differs between groups and repeats unchanged on each group's ID-1, ID-7 and CP. The declared neutral group's raw byte is nonzero. | A direct semantic charge-to-raw-byte cast, including neutral → zero, is disproved for this pair. The bounded fresh-modern rule adopted below supersedes this initial uncertainty. Formula spelling alone remains insufficient evidence. |
| Names and grouping | Equal native element coefficients with different declared charge occupy distinct ID-1 groups. The local function name is reused across those groups while formula labels differ; compound names remain empty. | A formula-qualified namespace is necessary to disambiguate these local function names. Optional unqualified SLN syntax does not establish how ambiguous references resolve. |
| IDs, counters, references | Both groups start their entry counters alike and advance in ID-1/ID-7/CP order. Their solid phase IDs match across groups; negative IDs and CP links are group-local. References remain zero. | This confirms group-local reset for two fresh groups, not allocation of a second phase inside one group or active-reference encoding. |
| Timestamps and CP | ID-1, ID-7 and CP share one timestamp within each group; the groups have distinct timestamps in the same file. Both observed nonzero-Cp ranges use ID-2. | Resolved bounded policy: use OLE Automation save/build time for changed groups and ID-2 for fresh zero-Cp as well as nonzero-Cp. The unchanged ID-9 date is not a per-group timestamp source. |

### Added negative-charge follow-up

A receipt-admitted fresh FDB now contains a third function group declared by
the user as charge `-1`. Matching the earlier groups by formula label plus
function name identifies one new group without relying on record order. The
new group's charge byte repeats on ID-1, ID-7 and CP. Provider parsing and an
independent byte-offset check agree that subtracting the observed neutral raw
offset from the new byte yields `-1`; the earlier neutral yields zero, and the
plus-marked group yields `+1` under the same arithmetic. No private formula or
coefficient is retained in this tracked assessment.

Domain policy confirms the neutral offset as the **native FDB charge rule** for
semantic charge `-50..=50`: write the native value as `charge + 50`, and decode
admitted raw bytes `0..=100` as `raw - 50`. The same byte is copied to ID-1,
ID-7 and CP records. `FdbChargeState` now exposes both checked conversions;
construction-plan validation rejects charge outside this native FDB range. All admitted
bytes fit the parser's `i8` slot, so changing the raw struct's signedness is
unnecessary. The observed groups establish the central states; extending the
linear rule across the admitted range is an explicit profile decision, not a
claim about every FactSage version or the Legacy translation profile.

The proposed eight-bit complement rule `charge_raw = !semantic_charge` also
fails: the declared neutral group's raw byte is not `!0`. If the bracket-plus
label denotes numeric `+1`, the charged group's byte is not `!1` either. An
independent byte-offset check agrees with the provider parser; the two observed
raw bytes differ in exactly one bit. That difference does not establish a
bitmask mapping. The native FDB offset rule is separate from this
rejected hypothesis.

This probe strengthens the bounded one-function-per-group structural profile,
but does not complete a native materializer. In particular, it establishes a
charge-sensitive raw distinction while falsifying the simplest numeric charge
mapping. Input-linked element encoding, timestamp generation and versioned
uninterpreted fields are still missing.

### Supplemental function-view screenshots

User-supplied screenshots show functions opened in FactSage 7.3's
**Solution** module, which displays both functions and solutions. They select
negative-marked, unmarked and positive-marked formula groups. Each tree group
shows one function; the selected functions have the same displayed name, range count
and visible thermodynamic inputs. This independently corroborates that the
function viewer displays separately qualified groups with a reused local
function name. The screenshots are not stored in the repository.

The window does not show a numeric signed-charge input, an elemental-composition
editor, native FDB fields, or the FDB creation action. The window title and
status-bar path do not make the displayed functions solution-only records. The
bracket-plus spelling is a displayed label, not independent proof of the
entered numeric charge. The visible application version identifies the viewer,
not necessarily the version that created the FDB snapshots. These screenshots
therefore do not change the native charge, element-mapping, versioned-header
or timestamp blockers.

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
relationships support the FDB-specific pseudo-solid Function-ID rule; generic
CDB aggregate-state bands are not projected onto FDB construction.

The emitted ID-5 A objects in this translation retain the source leading H
anchor and reverse the sign of nonzero source S. The tracked small pairs agree:
their only nonzero A source S also reverses, while zero S cannot distinguish
either sign. This supports a bounded constant ID-5 A anchor rule, not a rule
for ID-2 A objects with nonzero Cp or their unresolved source reduction.
Emitted objects are referenced by Modern SLN and omitted objects are not.
Physically zero Legacy base/A Function objects may be omitted in
`NativeFactSage`; the rigorous semantic graph may retain their identities.
The remaining uncertainty is the source-to-target scientific reduction for
nonzero Legacy objects, not the zero-object serialization policy.

For identity-linked base functions, counted source-range cardinality and
upper-temperature bounds transfer in encounter order across represented
models. Direct native H/S/Cp slot copying holds for a subset; other matched
base ranges have transformed H/S anchors or rearranged Cp slots. Their
like-power aggregate Cp bases agree in the bounded checked subset, so a slot
difference alone does not prove a different Cp function. The earlier
small-fixture conversion table remains a bounded observation, not a universal
Legacy-to-FDB record transform. The native builder can accept explicit target
H/S/Cp without solving the later source-to-plan adapter.

## CP-kind decision and fresh-write policy

FreshModern construction uses native ID-2 for every Cp range, including an
exact-zero Cp range. Preserve caller/function-definition range order and require
contiguous boundaries. Preserve every supplied coefficient/power pair exactly;
zero-fill only unused coefficient slots and the corresponding unused power slots.

LegacyTranslation is distinct: observed translated output uses ID-2 for nonzero
Cp and ID-5 for exact-zero Cp, but source-to-target H/S/Cp/A reduction can be
model/function dependent. That scientific conversion problem belongs upstream of
native materialization when the target plan has not already been made explicit.

## Remaining FDB-C3 gates

| Object or capability | Class | Exact missing rule or action |
| --- | --- | --- |
| ID-9 and per-kind uninterpreted bytes | RESOLVED_FOR_BOUNDED_CONSTRUCTION | Preserve the complete ID-9 record from one controlled empty template and copy per-kind opaque bytes from matching fresh record templates. Isolated ID-9 date/comment mutations failed in FactSage 7.3. |
| Shared charge | RESOLVED | FDB encodes semantic charge `-50..=50` as `charge + 50` and copies it through ID-1/ID-7/CP. This is source-profile independent and applies to both FreshModern and LegacyTranslation. |
| Shared composition | RESOLVED_FOR_BOUNDED_CONSTRUCTION | Parse with `chemformula`; preserve non-hydrogen encounter order and place hydrogen last; use atomic numbers as one-byte IDs; mirror ordinary integral stoichiometry into integer and `f64` arrays. Advanced real-stoichiometry overrides remain outside scope. |
| Formula and compound names | RESOLVED_FOR_BOUNDED_CONSTRUCTION | Preserve exact validated caller formula spelling; fresh `compound_name` defaults empty. Formula-qualified semantic identity remains separate from local Function naming. |
| Timestamp assignment | RESOLVED_POLICY | OLE Automation dates. New/materially changed groups use current save/build time consistently across ID-1/ID-7/CP; unchanged groups may preserve timestamps. |
| Function ID allocation | RESOLVED_POLICY | FDB Functions use group-local pseudo-solid IDs `101, 102, 103, ...` in Function encounter order regardless of semantic aggregate state; negative IDs are arithmetic negations and CP links use exact positive IDs. |
| References and density | PARTIALLY_RESOLVED | References default to zero. Ordinary density is supported using the low/remainder density portion with no advanced family code; advanced volumetric-family mappings are extensions and active nonzero references remain outside the bounded profile. |
| Fresh zero-Cp kind and shorter Cp term lists | RESOLVED_POLICY | Fresh ranges use ID-2, including zero Cp. Unused coefficient and power slots are written as zero. |
| Fresh empty-function verification | IMPLEMENTED | The writer emits ID-7 without CP and verifies strict reparse/domain structure; it skips ordinary Cp-backed evaluation for this case. |
| Legacy A conversion and zero-object omission | PARTIALLY_RESOLVED | Physically zero base/A Function objects may be omitted in `NativeFactSage`, while rigorous semantic identities remain internal. Matched constant ID-5 A objects have direct leading H, negated leading S and zero Cp after unit conversion. Eligible emitted A companions with counted ranges use observed native bounds of 298.15 K to the last ordinary upper bound. Combined nonzero-Cp A, zero-range A upper bounds and leading tails remain separate scientific work. The uncounted two-line A-leading block must not inherit ordinary counted-range semantics. |
| Provider materializer and verification pipeline | IMPLEMENTED_FRESH_AND_BOUNDED_LEGACY | `materialize_fresh` uses controlled templates, serializes, strict-reparses, indexes and checks group/function/CP identity plus Cp/H/S/G over each admitted range. Bounded Legacy output omits explicit zero A and selects ID-2/ID-5 for nonzero/zero Cp; ID-5 uses a caller-supplied native example. |
| Provider-built file acceptance | PASSED_CONTROLLED_OPEN_AND_ADDITIONAL_VISIBLE_FUNCTIONS | The corrected provider-built FDB opened, saved, and reopened in FactSage 7.3; the local FDB and SLN files were byte-identical before and after Save. Hydrogen-only, multiple/charged, and ID-2/ID-5 zero-added translated Functions opened visibly. One paired SLN/FDB probe showed its solution and endmember, and its `CpXp` term resolved internally; native thermodynamic evaluation remains unverified. |

The bounded FreshModern writer preserves the complete template ID-9 header and
has one native open/save/reopen acceptance plus visible Function checks for
hydrogen-only and multiple/charged groups. Legacy source-to-target scientific conversion remains a separate
upstream problem and does not block native serialization of an already explicit
FreshModern target plan.

## Next implementation/verification action

Verify native thermodynamic evaluation of an SLN reference to a constructed
Function, then further range/active-A shapes when their source-to-target
semantics are established.

The bounded `materialize_legacy_zero_added` method now emits an already
reduced LegacyTranslation target when every base has at least one homogeneous Cp range and
its paired A contribution is explicitly zero. It selects ID-2 for nonzero Cp
and ID-5 for exact-zero Cp, taking canonical zero-Cp power slots from a
caller-supplied translated native example. It omits zero A records, strictly
reparses, and uses the same identity and thermodynamic checks. Empty-range
bases, mixed ID-2/ID-5 ranges, active A, and SLN bundle closure remain typed refusals or separate
gates; this does not generalize the raw Legacy G reduction. A generated ID-5
probe opened with its Function visible in FactSage 7.3.

No additional user/domain decision is required to continue this engineering
and verification work.


### Historical `dbsolution` evidence for Legacy added-leading records

Historical `evnekdev/dbsolution` code provides implementation evidence that the
uncounted leading two-line record and the counted ordinary ranges were treated
as different grammars:

- `parse_range` reads an ordinary counted range as H/S, four fixed Cp fields,
  explicit upper temperature, density, then three variable coefficient/power
  pairs plus four magnetic fields;
- `parse_range_added` marks a distinct added Function/range, reads H/S plus
  four leading fixed fields, assigns `TMIN = 298.15` and `TMAX = 6000.0`
  internally rather than reading ordinary range bounds, then reads the second
  line as two 15-character unknown values followed by four 10-character unknown
  values;
- `write_range_added` likewise emits a separate fixed added-record shape and
  writes zeros for those six trailing fields rather than using the ordinary
  variable-Cp/magnetic writer.

This is historical parser/writer evidence, not proof that every provisional
field name used by that old code is scientifically correct. In particular, the
old code calling the first two values `H` and `S` does not by itself establish
that the second leading value is universally an FDB entropy anchor. However, it
does establish that the added leading pair must **not** inherit ordinary
`xxxx` range field semantics position-by-position.

Consequently, Legacy conversion must model the uncounted added-leading pair as a
distinct raw source structure and derive any target `xxxxA` Function/ID-5
thermodynamics from evidence, rather than reusing the ordinary range decoder.

The sign reversal seen for nonzero source second fields in constant A targets
also supports a possible Gibbs-energy interpretation: the source value may be a
linear-in-temperature coefficient whose thermodynamic entropy is its negative
derivative. That was a hypothesis before the controlled imports below. An
earlier derivative test of conventional Gibbs terms and variable powers did
not match the admitted coupled nonzero-Cp A targets
under the tested slot assignments and temperature bases. Keep nonzero-Cp A
construction blocked until its actual source basis and any matrix-level
reduction are established.
An additional fit retained the source coefficient order and allowed a distinct
constant multiplier for each leading, fixed and variable slot. That conventional
basis still failed the fit subset for combined H/S/Cp and for the H/S-only and
Cp-only checks; some multipliers were not identifiable. Per-slot scaling alone
does not establish this candidate as the conversion rule.

**ConfirmedByDomainExpert:** `Gadded` applies to all counted ranges of its G
entry simultaneously. Controlled nonmagnetic and magnetic SUBL imports now
establish isolated Gibbs responses for first leading line slots 1, 2, 3 and
6. The controlled added target reaches the ordinary entry's final upper bound.
An identity-linked audit of eligible paired translations supports native A
bounds from 298.15 K to the last counted ordinary upper bound. Many ordinary
bases start earlier. Zero-range A entries share the observed 298.15-K lower
bound but have varying upper bounds and remain outside this rule.
Nearest preceding/following counted upper bounds and solution-wide extrema
are not universal zero-range A upper-bound rules in the paired corpus.
First-line slots 4–5 and second-line slots 1–6 remain without a general
analytic mapping; magnetic activation did not change their isolated responses.
The Legacy-to-Modern plan may provisionally zero those unresolved leading
positions, preserve the raw source, and flag suppressed nonzero values. This is an
explicit approximation; exact Legacy A construction and zero-range A upper bounds
remain blocked.

The solution provider now exposes a bounded provisional scientific A target:
with caller-established bounds containing 298.15 K, it derives H/S anchors and
`Cp(T) = -c_TlnT - 2 c_T2 T` from the four retained Gibbs terms and carries a
suppressed-slot loss flag. It checks finite thermodynamics at both bounds. A
counted-range helper applies the observed 298.15-K-to-last-upper-bound profile.
This target does not yet allocate active native A records or establish the
ID-7 phase anchors, CP-kind rule for combined active terms, or physical tails.
The bounded Legacy zero-A writer is GO; only active-A and other out-of-profile
LegacyTranslation extensions remain blocked on those policies and acceptance.

Large paired-file differential verification now supports up to nine homogeneous
ordinary Cp ranges in the zero-A LegacyTranslation route. Across 296 matched
zero-A Functions with inactive per-range auxiliary fields, source and paired
native output agree on range count, bounds, canonical Cp basis, and analytically
reanchored H/S through eight ranges. Other Functions in the paired native file
establish the repeated CP-record shape through nine ranges. The generated nine-range native stream passes strict reparse
and domain indexing; a controlled FactSage GUI open of this new nine-range
output remains a separate acceptance check. The direct FreshModern profile
retains its three-range limit.


**Paired unit-scaling evidence:** the leading pair is not merely an ordinary
range with an omitted count. In the tracked paired fixtures, an ordinary
counted range's first field is the legacy `H / 1000` representation and maps
to FDB enthalpy with `×4184` J/mol. The added leading pair's first field maps
to the emitted `xxxxA` ID-7 and ID-5 enthalpy anchors with `×4.184` J/mol.
That factor-of-1000 difference independently proves distinct field semantics
for at least the first position. The second leading value is zero in 13/14
tracked emitted A blocks, so those matches do not distinguish direct copying
from sign reversal. The sole nonzero tracked case and the admitted larger
translation instead agree with `S_target = -4.184 × S_source` for matched
zero-Cp ID-5 A objects. Their interval bounds, leading-tail fields and
nonzero-Cp A counterparts still need a separate reduction rule.
