# Bounded fresh-modern FDB construction profile

**Status:** normative implementation policy for the first provider-owned fresh FDB writer.

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
- one ordinary solid function per formula group;
- one to three nonzero-Cp ranges;
- no automatic A companion;
- inactive magnetic/PV/transition/ID-11 physics;
- no active references;
- no active density;
- ordinary integer formula stoichiometry;
- no advanced real-stoichiometry override;
- semantic charge `-50..=50`;
- caller-provided function name;
- caller-provided database comment and OLE Automation date.

Features outside this profile remain explicit capability exclusions, not blockers
for this profile.

## ID-9 database header

Do **not** reverse-engineer opaque ID-9 fields for this profile.

Use the ID-9 record from the controlled empty fresh FDB as the versioned native
template. Preserve all unknown/reserved/padding/read-flag bytes byte-for-byte.

Only fields whose meaning is established may be regenerated:

- `date_ole`: OLE Automation date (`f64` day count);
- `comment`: native 80-byte database comment.

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
2. preserve element first-appearance / parser encounter order as the native slot
   order;
3. encode each real element ID as its periodic-table atomic number in one `u8`;
4. align integer and real coefficient slots with those element slots;
5. verify this rule against the existing local fresh and translated FDB corpus as
   an implementation regression check.

**Domain policy confirmed by the user:** equivalent formulas are not reordered or
canonicalized for native element-slot construction. The native slot order follows
the order in which elements first appear in the parsed formula. Corpus checking is
verification of the implementation, not a remaining user-input blocker.

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

For the bounded fresh-modern profile:

`raw_charge = semantic_charge + 50`

with semantic charge restricted to `-50..=50`, yielding raw bytes `0..=100`.
Copy the resulting byte consistently through the ID-1, ID-7 and CP shared
headers.

The observed fresh `-1, 0, +1` cases support the central mapping. Extending it
over the admitted range is the explicit bounded profile policy. This does not
claim the same encoding for Legacy translation or every historical database.

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
fresh evidence shows they can differ. Exact FactSage clock/rounding behavior is
not a human-in-the-loop blocker for the bounded profile.

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
- compound-name field may remain empty, matching the controlled fresh evidence;
- local function names may repeat across distinct formula/charge groups;
- formula-qualified identity disambiguates such groups;
- an empty SLN formula qualifier does not mean the FDB object lacks composition.

## Other native defaults

For per-record unknown, reserved and padding fields whose semantics remain
opaque, use the values observed in the controlled fresh profile/template for the
corresponding record kind and validate them by independent FactSage acceptance.

This is a versioned serialization policy. It is preferable to inventing meanings
for opaque bytes.

For the first profile, references are written as zero by default. These are
internal FactSage ecosystem references and are not required for ordinary fresh
function construction. Active/nonzero reference encoding remains outside scope
unless explicitly requested later.

Inactive physical metadata is admitted. Active density remains outside scope.

## Cp policy

For the first profile:

- admit nonzero Cp only;
- use native ID-2, which is the observed direct-modern form for all fresh
  nonzero-Cp ranges;
- preserve range order and chained bounds;
- require a term shape already supported by the provider/native slot layout
  (initially seven/eight stored slots as appropriate);
- use observed zero/default padding for unused native bytes;
- reject fresh zero-Cp construction until the later policy/acceptance path is
  implemented.

Fresh zero-Cp, ID-5 selection and Legacy A reduction are therefore not blockers
for this bounded profile.

## Materialization and verification gate

The provider may now implement:

`FdbBuildPlan -> bounded-profile validation -> RawDatabase -> serialize ->
strict reparse -> DomainIndex -> thermodynamic verification`.

After internal verification, the generated tiny FDB must be opened by the
corresponding FactSage version and survive load/open/save without changing the
intended function values or references.

Independent FactSage acceptance is a **verification gate**, not a request for
new domain knowledge. It should be automated from the local development
environment where possible.

## Blocker interpretation

For this bounded profile, the following older blocker categories are superseded
by the policies above:

- ID-9 opaque/default initialization;
- fresh charge encoding;
- timestamp format;
- ordinary element-ID mapping;
- ordinary integer/real stoichiometry default;
- fresh compound-name omission;
- singleton solid phase allocation;
- zero-reference/inactive-density policy;
- nonzero-Cp ID-2 selection.

The current `native_blockers()` implementation may still report some of these
legacy evidence requests until the implementation tranche reconciles the blocker
API with this profile. That reconciliation is engineering work and must not be
reported as a human-in-the-loop requirement.
