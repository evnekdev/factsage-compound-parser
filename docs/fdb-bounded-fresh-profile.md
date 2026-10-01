# Bounded fresh-modern FDB construction profile

**Status:** bounded writer implemented; a revised provider-built one-function
FDB opened, saved, and reopened in FactSage 7.3 with byte-identical local files.
Hydrogen-only and multiple/charged FreshModern Functions and ID-2/ID-5
zero-added LegacyTranslation Functions also opened visibly.

This profile deliberately avoids solving every historical CDB/FDB behavior before
constructing a useful modern FDB. It captures the domain decisions and bounded
native policies established during the `data/v0` / `data/v1` audit. It is a
construction profile, not a claim that every FactSage version uses the same
encoding in every database family.

## Human-in-the-loop status

There are **no remaining domain-policy or scientific questions that require user
input before implementing this bounded profile**.

Remaining work is engineering and verification:

1. implement the profile;
2. automatically check its inferred ordering/default rules against the existing
   ignored local evidence;
3. serialize and strict-reparse the generated FDB;
4. verify the provider/domain/thermodynamic view;
5. run independent FactSage acceptance and load/open/save stability.

If an automated check falsifies one of the bounded rules below, report the exact
counterexample and narrow the implementation. Do not request a new user decision
unless the evidence demonstrates a genuinely new semantic choice.

## Admitted scope

The first writer may intentionally admit only:

- direct `FreshModern` construction;
- multiple ordinary or ID-7-only empty functions per formula group; native Function IDs use the FDB pseudo-solid series regardless of semantic aggregate state;
- zero to three Cp ranges; zero-Cp ranges use the normal fresh ID-2 representation;
- no automatic A companion;
- inactive magnetic/PV/transition/ID-11 physics;
- no active references;
- ordinary density may be supplied; advanced volumetric equation families are optional extensions;
- ordinary integer formula stoichiometry;
- no advanced real-stoichiometry override;
- semantic charge `-50..=50`;
- caller-provided function name;
- an unchanged ID-9 date and empty comment from one controlled empty FDB;
  caller-supplied record timestamps remain separate.

Features outside this profile remain explicit capability exclusions, not blockers
for this profile.

The separate zero-A `LegacyTranslation` materializer admits up to nine
homogeneous ID-2 or ID-5 ranges per Function. A content-free differential audit
of the large paired Legacy/Modern evidence found exact range counts, bounds,
Cp basis, and reanchored H/S for all 296 matched zero-A ordinary Functions
with inactive per-range auxiliary fields; these direct matches span one through
eight ranges. Other Functions in the paired native file establish the repeated
CP-record shape through nine ranges. A synthetic nine-range
translated file strictly reparses and indexes; native FactSage acceptance of
the newly generated nine-range shape is still pending.

Translated plans may explicitly preserve two differently scaled formula units
with the same reduced stoichiometric ratio under distinct native labels. The
dedicated translated constructor still rejects an exact duplicate composition
and charge. A synthetic two-group file strictly reparses and indexes. Direct
FreshModern construction retains ratio-based uniqueness.

The same paired translated evidence contains formula labels with a zero
coefficient element absent from their nonzero semantic composition. Its native
ID-1 header retains the element ID in a zero-coefficient slot. The translated
materializer now retains such label-only zero slots and verifies them after
strict reparse. FreshModern continues to reject this spelling outside its
controlled profile.

## ID-9 database header

Do **not** reverse-engineer opaque ID-9 fields for this profile.

Use the ID-9 record from the controlled empty fresh FDB as the versioned native
template. The one-range exemplar must carry the identical ID-9 record. Preserve
the complete record byte-for-byte, including its date and empty comment.
`materialize_fresh` rejects a plan that asks to change either field. In a
controlled FactSage 7.3 test, changing only the native ID-9 date or only its
comment caused rejection; changing only entry timestamps did not. The admitted
fresh snapshots likewise retained one ID-9 date and empty comments. A broader
database-creation or paired-SLN date/comment policy remains outside this
bounded profile.

With the complete native header and composition retained, FactSage 7.3 opened
a probe carrying generated thermodynamic fields and function name. A probe
with a generated composition and the same native header opened but displayed
no function while hydrogen occupied the first element slot. Moving hydrogen
last made the Function visible; the corrected Rust writer then generated an
FDB that FactSage opened, saved, and reopened. The local FDB and SLN files
remained byte-identical after Save. Broader output shapes remain unverified.

No dedicated database-name field is currently established in ID-9. Do not invent
one. Database/file naming remains outside the opaque header unless later evidence
shows an explicit field.

Template provenance remains local/confidential. Tracked code/tests may contain
only synthetic or generalized representations; do not commit private FDB bytes.

## Formula parsing and native composition

Use the `chemformula` crate as the canonical formula parser for this path. It
was designed for the ChemApp/FactSage-style thermochemical formula domain and
already provides:

- nested formula parsing;
- charged formula parsing;
- ordered element/coefficient pairs;
- periodic-table element identities;
- atomic-number/index lookup.

For the bounded profile:

1. parse the formula with `chemformula`;
2. preserve parser encounter order among non-hydrogen elements, then place
   hydrogen in the last occupied native slot when present; retain the exact
   caller formula label. Controlled FactSage 7.3 probes showed that a
   hydrogen-first group opened without a visible Function, while the same
   label and composition with hydrogen last displayed its Function. All
   hydrogen-containing groups in the admitted native FDB corpus also put
   hydrogen last. A provider-built hydrogen-only Function also opened visibly
   in FactSage 7.3;
3. encode each real element ID as its periodic-table atomic number in one `u8`;
4. align integer and real coefficient slots with those element slots;
5. verify this rule against the existing local fresh and translated FDB corpus as
   an implementation regression check.

The earlier first-appearance policy remains in force among non-hydrogen
elements. Controlled FactSage behavior contradicted it for hydrogen-first FDB
groups, so the bounded writer moves hydrogen to the last slot without changing
the formula label or semantic composition. This revision is specific to FDB
Function construction; it does not establish a CDB-wide ordering rule.

Vacancies and phase electrons are outside this first ordinary-composition profile
unless the native FDB encoding is already independently established.

## Stoichiometric coefficients

For ordinary integer formulas, use the parsed formula coefficients as the native
integer coefficients.

The native real-stoichiometric coefficients are an advanced compound-database
feature that can represent an actual composition different from the compact
integer formula (for example, a nominal integer formula with a non-integral real
stoichiometry). This is not required for the first FDB profile.

Default bounded policy:

- integer coefficient slots = parsed integral formula coefficients;
- real coefficient slots = the same coefficients converted to `f64` (`real_stoichiometric_coefficients` is native `f8[7]` / `[f64; 7]`);
- advanced real-stoichiometry overrides are rejected/not admitted initially.

Existing simple FDB evidence already shows integer-formula cases where the integer
and real arrays agree with the formula amounts. The implementation must
automatically confirm this against the local corpus.

## Charge

FDB uses the native charge encoding:

`raw_charge = semantic_charge + 50`

with semantic charge restricted to `-50..=50`, yielding raw values `0..=100`.
Copy the resulting value consistently through the ID-1, ID-7 and CP shared
headers.

This is a property of the FDB representation, not of the source construction
profile. FreshModern and LegacyTranslation therefore use the same mapping once
semantic charge is known. The observed fresh `-1, 0, +1` cases provide direct
corpus confirmation around the neutral offset.

## Timestamps

The database date and shared-entry timestamps are OLE Automation dates stored as
little-endian `f64` day counts. The crate already has `OleAutomationDate`
support.

Timestamp generation is therefore a writer policy, not a native-format blocker.

**Domain policy confirmed by the user:** timestamps are mostly ignored metadata.
When a group is created or materially changed, use the current save/build time
encoded as an OLE Automation date for that changed group. Preserve timestamps on
unchanged groups when editing an existing database. Within a newly written or
changed group, copy the same timestamp consistently through its ID-1, ID-7 and
CP records, matching the observed fresh-modern pattern.

Do not require the ID-9 database date to equal every per-group timestamp; the
fresh evidence shows they differ. Keep the controlled ID-9 date unchanged while
assigning a timestamp to newly written entry groups. Exact FactSage
clock/rounding behavior is not a human-in-the-loop blocker for the bounded
profile.

## Function/group names

Fresh-modern function names are caller supplied. Do not impose Legacy
`<FILE phase id>_<NNNN>[A]` naming.

The FDB formula label, semantic composition/charge identity, function name and
SLN formula qualifier are distinct concepts.

For the bounded fresh profile:

- formula label is the caller-supplied formula spelling exactly as entered;
- validate that spelling with `chemformula`;
- if parsing fails, reject it and require a valid formula rather than silently repairing or canonicalizing it;
- preserve the entered element order in the stored label and in native composition slots (for example, `O4Fe3` remains `O4Fe3`);
- **confirmed domain policy:** leave the ID-1 `compound_name` field empty by default for fresh-modern construction, matching the controlled fresh evidence; translated FDB alias behavior does not change this fresh-writer default;
- local function names may repeat across distinct formula/charge groups;
- formula-qualified identity disambiguates such groups;
- an empty SLN formula qualifier does not mean the FDB object lacks composition.

## Other native defaults

For per-record unknown, reserved and padding fields whose semantics remain
opaque, copy the corresponding values from the observed controlled fresh FDB
record template for that native record kind.

**Confirmed domain policy:** do not infer meanings for these fields in the
bounded writer. Treat them as versioned template bytes and validate the resulting
records by independent FactSage acceptance.

This is a versioned serialization policy and supersedes older evidence requests
for semantic interpretation of opaque fields.

For the first profile, references are written as zero by default. These are
internal FactSage ecosystem references and are not required for ordinary fresh
function construction. Active/nonzero reference encoding remains outside scope
unless explicitly requested later.

Density is a common optional property and is not inherently outside scope.

**Domain clarification confirmed by the user:** the high-order packed portion of
`density_raw` encodes the selected advanced volumetric-property equation family.
The low/remainder portion carries the physical density value. FactSage can pack
the family code into the same numeric field because physically realistic density
values are bounded well below the encoding scale.

For the bounded writer:
- if no density/advanced volumetric model is supplied, use the observed zero/default
  inactive representation;
- if density is supplied without an advanced volumetric model, write the ordinary
  density value with the default/no-advanced-family code;
- if an advanced volumetric model is supplied, use the corresponding family code
  and coefficients once that family's native mapping is implemented.

Advanced volumetric equation-family mappings are capability extensions, not a
blocker for ordinary density support.

## Cp policy

For the first profile:

- use native ID-2 for fresh Cp ranges, including zero-Cp ranges;
- represent zero-Cp by writing zero to all Cp coefficient slots and zero to all
  corresponding power slots;
- preserve range order and chained bounds;
- require a term shape already supported by the provider/native slot layout;
- write zero to every unused Cp coefficient slot and zero to the corresponding
  unused power slot;
- use observed zero/default padding for other unused native bytes;
- do not use a separate fresh zero-Cp record kind in the bounded profile.

Fresh zero-Cp, ID-5 selection and Legacy A reduction are therefore not blockers
for this bounded profile.

## Materialization and verification gate

The provider may now implement:

`FdbBuildPlan -> bounded-profile validation -> RawDatabase -> serialize ->
strict reparse -> DomainIndex -> thermodynamic verification`.

After internal verification, a generated tiny FDB was opened in FactSage 7.3
and survived load/open/save with byte-identical local FDB and SLN files.
Additional hydrogen-only, multiple/charged, and translated zero-added ID-2/ID-5
probes displayed their Functions. A paired synthetic SLN/FDB probe displayed its
solution and endmember, and its one active `CpXp` term resolved uniquely against
the generated Function in the bundle resolver. Native thermodynamic evaluation
and broader reference shapes remain verification work.

Independent FactSage acceptance is a **verification gate**, not a request for
new domain knowledge. It should be automated from the local development
environment where possible.

## Blocker interpretation

For this bounded profile, the following older blocker categories are superseded
by the policies above:

- ID-9 opaque/default initialization;
- FDB charge encoding;
- timestamp format;
- ordinary element-ID mapping;
- ordinary integer/real stoichiometry default;
- fresh compound-name omission;
- singleton solid phase allocation;
- zero-reference/inactive-density policy;
- nonzero-Cp ID-2 selection.

`native_blockers()` reports remaining native coverage after the successful
one-function open/save probe and visible Function checks; it does not reopen
the resolved native field policies.

The internal post-serialization check now compares reparsed group composition,
charge, formula, Function names/IDs, phase anchors, and CP links/bounds with the
typed plan as well as sampled H/S/Cp/G. Synthetic multi-function/charged and
hydrogen-only probes pass that check and their Functions opened visibly in
FactSage 7.3. One synthetic SLN reference resolved internally and its solution
and endmember displayed natively; calculation with that reference remains a gate.


## Advanced volumetric coefficients

For now, treat the physical-property coefficient blocks as a multipurpose native
coefficient array whose detailed interpretation depends on the packed volumetric
equation-family selector. Do not invent per-family semantics in the bounded
writer.

Before adding named advanced volumetric families, inspect the historical Legacy
Python parser for existing family definitions/mappings. If they are absent or
insufficient, later user-provided snapshots may be used to establish them.
This does not block ordinary density or the first bounded fresh writer.


## Phase-state native ID allocation

Generic CDB parsing uses native aggregate-state bands (solid 100+, liquid 800+,
gas 900+, aqueous 990+), but that behavior must not be projected onto FDB
construction.

**Confirmed bounded FDB policy:** treat FDB Function records as a pseudo-solid
series within each formula group and allocate consecutive raw IDs `101, 102,
103, ...` in function encounter order, irrespective of the source
solid/liquid/gas/aqueous state. This matches the observed fresh and translated
FDB corpus, where all Function records use the solid band.

For ordinary Function records, write `phase_id_raw_neg = -phase_id_raw`.
CP links use the exact positive raw Function ID. Reset the sequence per formula
group.

If future FDB evidence shows use of another state band, add that as an explicit
profile extension rather than inheriting generic CDB phase-state semantics.


## Entry-number allocation

Observed fresh-modern and translated FDB evidence agrees on the bounded rule:
each formula group starts from the same native group-start value, then
`entry_number` increments by one for every emitted ID-1, ID-7 and CP record in
physical stream order. A new formula group restarts from the group-start value.

For construction, obtain the native group-start value from the controlled fresh
record template and apply consecutive increments. The bounded writer rejects a
group before byte rollover rather than inventing wraparound semantics.

This rule is confirmed by both fresh and translated FDB observations and is not
a human-in-the-loop blocker.


## Function/CP physical stream order

**Confirmed parser/native ordering rule:** within one formula group, all
Function (ID-7) records precede all CP/ID-11 records:

`ID-1 group header -> ID-7 function 1 -> ID-7 function 2 -> ... -> CP/ID-11 records -> ...`

This ordering is enforced by `DomainIndex`: once a CP/ID-11 range has occurred,
a later Function record is a fatal ordering error. Function records retain caller
encounter order. CP records retain their owning function/range order and link to
the exact positive Function ID. Entry numbers advance in the actual emitted
physical stream order.


## Cp range ordering and continuity

**Confirmed domain policy:** preserve Cp ranges in caller-supplied/function-definition
order. For a multi-range function, require contiguous temperature boundaries:
each range's `temperature_max_k` must equal the next range's
`temperature_min_k`.

Reject overlaps, gaps, or reordered ranges in the bounded writer rather than
silently sorting or repairing them.


## Empty fresh Function policy

**Provisional bounded policy:** for a fresh Function with no Cp ranges, emit the
observed native empty form: one ID-7 Function record and no CP record.

This matches the controlled fresh-modern snapshot already present in the local
evidence. Do not synthesize an artificial zero-Cp interval merely to satisfy a
thermodynamic evaluator. Structural verification for this case should use
strict reparse/domain checks rather than an ordinary H/S/Cp view that requires a
range.

This is a bounded implementation choice and remains subject to FactSage
acceptance; no additional human decision is required unless that acceptance
fails.


## Legacy-translated zero-object omission

**Confirmed domain policy:** when serializing Legacy-translated FDB content,
physically zero base/A Function objects may be omitted in the native
`NativeFactSage` representation, matching observed FactSage behavior.

The rigorous semantic layer may still retain those identities for provenance,
dependency accounting, and exact source reconstruction, but the native FDB
serializer does not need to emit zero-valued Function records merely to preserve
the internal semantic graph.

Any SLN/native references must remain consistent with the emitted native object
set. This is a serialization policy, not a change to the rigorous semantic
identity model.
