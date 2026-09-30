# Fresh FDB field inventory (FDB-C1)

This inventory distinguishes directly authored fresh-modern FDBs from the
rigorous pure H/S/Cp Legacy → Modern Function profile.
It is a construction inventory, not a recipe for writing native bytes. The
physical field list is from `src/raw/*` and `docs/chunk-layouts.md`; provider
meaning is from `docs/thermodynamic-semantics.md` and the paired Legacy G/FDB
audit on `factsage-solution-parser` branch
`architecture/legacy-modern-conversion-contract`. Database Compare's
conversion-blocker contract controls the quality of the remaining questions.
No private database bytes are included here.

Classification vocabulary: `CALLER_PROVIDED`, `PROVIDER_GENERATED`,
`EVIDENCED_DEFAULT`, `NOT_REQUIRED_FOR_PROFILE`, `RAW_PRESERVED_ONLY`, and
`UNRESOLVED_BLOCKER`. A field marked unresolved is **not** assigned zero by
construction. Evidence tags: `CODE` = provider parser/domain/thermo code;
`PAIR` = tracked paired Legacy/FDB audit; `CONTRACT` = cross-repository domain
contract; `UNKNOWN` = no sufficient fresh-construction evidence. A parsed zero
alone never establishes a construction default.

## FDB-C2 semantic identity and evidence level

The semantic key is **canonical exact elemental ratios plus explicit signed
charge**. `FdbStoichiometricAmount` holds a positive rational with `u64`
numerator and denominator; the provider sorts element symbols and reduces
each ratio against the first element using `u128` products and GCD. Caller
element order, common scale, decimal rendering and `f64` bit patterns cannot
change this key. The plan rejects more than seven distinct elements because
the native shared header has seven slots.
`FdbFormulaGroupPlan::semantic_identity()` exposes the key.
The formula label remains a separate caller-supplied target text field.
Duplicate labels and duplicate full keys are rejected. Different labels for
one full key are rejected by the database-global grouping **domain contract**,
not by a claim that every existing native FDB has this invariant. Different
charges may share elemental ratios and remain separate groups.
The target Function name is unique within a formula group; two distinct
full groups may carry the same local Function name because external references
are qualified by the formula label. One source G-entry role cannot be reused
across groups.

| Charge fact | Evidence level | Limit |
| --- | --- | --- |
| Every non-database raw record has `RawCommonHeader.charge_raw: i8`; parser and serializer preserve it. | CONFIRMED_BY_PROVIDER_CODE | The fresh-modern plan now has a bounded conversion rule; the raw parser remains lossless. |
| Formula label and composition are separately stored in ID-1; shared headers occur on ID-1, ID-7 and CP records. | CONFIRMED_BY_PROVIDER_CODE_AND_LOCAL_PAIR | The paired FDB repeats its raw charge within each group; this does not prove semantic charge encoding or a fresh constructor rule. |
| `Fe3O4` and `Fe3O4[+]` must remain distinct groups. | CONFIRMED_BY_DOMAIN_CONTRACT | Label syntax is not parsed as semantic charge. |
| Solution parser models source charge as `i32` and keeps bracketed charge spelling in external references. | CONFIRMED_BY_PROVIDER_CODE | Its SLN parser test is synthetic, not paired FDB evidence. |
| Historical `dbsolution` parses Legacy charge and derives SLN endmember charge from `chemformula`. | HISTORICAL_PYTHON_HINT | This does not prove FDB `charge_raw` initialization. |
| A user-declared fresh neutral/charged pair has equal native element IDs and integer/real coefficients but different `charge_raw`; the declared neutral raw byte is neither zero nor the eight-bit complement of zero. | CONFIRMED_BY_LOCAL_FRESH_PAIR | Direct cast and bitwise-complement encoding are refuted for this pair. The bytes differ in one bit, but the exact signed magnitude and native code mapping remain unknown. |
| A later fresh group declared as charge `-1` fits subtraction of the observed neutral raw offset; the earlier unmarked and positive-marked groups fit zero and conditional `+1`. | CONFIRMED_BY_LOCAL_FRESH_TRIPLE | The user adopts `raw = charge + 50` for fresh-modern semantic charge `-50..=50`; extending the linear rule beyond observed states is a bounded profile policy, not a universal format claim. |

Exact rational identity is a bounded semantic profile, not a claim that every
FactSage source coefficient has a `u64` rational representation. A future
solution adapter must supply an exact source fraction or declare a separate
conversion policy; it must not silently approximate an `f64` with a tolerance.
Native ID-1 integer/real coefficient encoding remains a separate evidence gate.

## Local paired-translation audit (FDB-C3)

The ignored local `data/v1` corpus contains a receipt-admitted Legacy solution
and Modern FDB/SLN translation. A local-only correspondence map linked source
G entries by source solution and encounter index, then checked target names and
SLN references. Multiple solution model families, charged groups, ordinary
and added functions, and zero-object cases were examined. No private names,
coefficients, native bytes, or per-object inventory are included here.

The observed base/A names use the FILE phase ID and zero-based, four-digit G
encounter suffix. Matching matrix coefficients transfer to Modern external
references without conversion or value-based merging. The FDB also contains
functions that do not correspond to a parsed source G entry, so the naming
rule is bounded to matched translated entries. Formula and compound names
differ in some groups, and external references can resolve through either.

The local translation narrows native rules without establishing fresh creation
defaults. Solid phase IDs increase in group stream order and restart within
each group; the negative field is the arithmetic negative of the positive ID
in the paired FDB and comparison CDBs. CP phase links are group-local. Shared
composition, charge, coefficient padding, and references repeat within an FDB
group. In the paired FDB, entry numbers advance once per ID-1/ID-7/CP record
in group stream order. CP timestamps copy the ID-1 group timestamp in the
paired FDB and both comparison CDBs; ID-7 timestamps can differ. Strict simple
integer FDB labels match atomic-number element IDs and matching integer/real coefficients
in label order, with empty slots zero; this pattern does not hold in the two
comparison CDBs and cannot be generalized to fractional or charged labels.

Base/A role does **not** select CP ID-2 versus ID-5. Across every CP record in
the paired FDB, ID-2 has at least one exactly nonzero Cp coefficient and ID-5
has all-zero Cp coefficients. The same ID-2/ID-5 split occurs in both local
comparison CDBs, although those CDBs also have zero-Cp ID-3 records.
Identity-and-order-linked counted base source ranges with nonzero Cp map to
ID-2, and all-zero Cp ranges map to ID-5 across represented model families.
This is a strong, falsification-checked **output invariant**, not yet a
fresh-construction rule: some added/generated records
need a source-to-target reduction before their Cp content is known.

Zero base/A objects can be emitted or omitted. Emitted objects have Modern SLN
references and omitted objects do not, but that output relationship does not
establish a source-side emission policy. The full local translation's emitted
ID-5 A anchors show direct H and opposite-sign S for nonzero S, whereas the
earlier small paired examples mostly showed direct S. A universal A sign rule
is therefore unsupported. Native materialization remains blocked even for a
bounded fresh profile.

## Direct-modern audit and profile boundary

The ignored receipt-admitted `data/v0` sequence contains fresh-modern empty,
empty-function, single-range, reference/density and three-range snapshots. It
is direct-modern construction evidence, while `data/v1` is translation
evidence. The fresh sequence has one named solid function, no hidden A
companion, and an empty ID-1 compound-name field with a populated formula
label. Its nonzero-Cp ranges all use ID-2. The empty function has ID-7 without
CP. Multiple ranges have separate CP records, common phase link, chained
bounds and stream-order entry numbers. The density/reference edit changes
phase density and phase references, not group or CP references. ID-1, ID-7 and
CP timestamps agree within a fresh snapshot, whereas translated ID-7 may
differ. ID-9 date stays fixed through the fresh edits while entry timestamps
change. The later fresh charged-function probe supplies two singleton solid
groups with equal native element arrays and differing raw charge. The original
neutral function's CP upper bound also changes between snapshots, so the
cross-snapshot comparison is not a pure add-only edit. Within the new file,
the two groups differ only in raw charge, formula label and per-group timestamp;
their local function names are equal. Their phase IDs and entry starts reset
to the same values, while shared composition/charge copy within each group.
The exact signed UI charge magnitude, input stoichiometry and version are not
declared. These observations do not establish a universal version default or
a second-function allocation rule inside one group.

User-supplied screenshots show functions opened in FactSage 7.3's combined
Solution module. Negative-marked, unmarked and positive-marked groups each have
one same-named function and equal visible thermodynamic inputs. They corroborate separate UI
groups and reused local function names; the window title and status-bar path
do not make these solution-only records. The screenshots do not show numeric
charge entry, the composition editor, native FDB fields, FDB creation, or the
FDB writer version; no native field mapping is promoted from label spelling.

`FdbBuildPlan::new` retains translated naming and explicit base/A pairing.
`new_fresh_modern` accepts caller-supplied names without invented Legacy
provenance or automatic A companions. An SLN reference may omit its formula
qualifier; that syntax does not erase FDB group composition, compound name,
formula label or function name.

## ID-9 database header

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| chunk ID | PROVIDER_GENERATED | `CODE`: ID 9 and first-record ordering are fixed. |
| `magic` | EVIDENCED_DEFAULT | `CODE`: parser requires `CMPD`. |
| `date_ole` | CALLER_PROVIDED | `CODE`: OLE date field; plan uses the existing representable-date validator. Whether native writer should use a particular clock source is a separate policy. |
| `read_flag` | UNRESOLVED_BLOCKER | `V0`/`V1`: zero throughout fresh edits and the translation; named-version initialization and provider-written acceptance remain unproved. |
| `comment` | CALLER_PROVIDED | `CODE`: 80-byte fixed ASCII; empty is explicit and allowed. |
| `padding_1`, `padding_2`, `padding_3` | UNRESOLVED_BLOCKER | `V0`/`V1`: all are zero across observed fresh and translated FDBs; this is a bounded observed pattern, pending versioned writer acceptance. |
| `unknown_1`, `unknown_2` | UNRESOLVED_BLOCKER | `V0`: both remain stable through edits and contain nonzero bytes. `unknown_1` agrees with the translated FDB, `unknown_2` differs. Version/source policy is unknown. |

There is no native database name/identifier field in this header. A filename or
logical bundle role belongs to orchestration (`NOT_REQUIRED_FOR_PROFILE` as a
header field). No invented database ID is in the semantic plan.

## Shared entry header (ID-1, ID-7, ID-2 and ID-5)

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| `element_ids[7]` | UNRESOLVED_BLOCKER | `V0`: repeat on ID-1/ID-7/CP and remain identical across the declared neutral/charged groups. `V1`: strict simple integer labels matched atomic-number IDs. Input-linked fresh element ordering remains unproved. |
| `element_coefficients[7]` | UNRESOLVED_BLOCKER | `V0`: group-wide repetition and equality across the neutral/charged groups. `V1`: bounded integer-label matches. Fresh input-to-slot scale remains unproved. |
| `coefficient_padding` | UNRESOLVED_BLOCKER | `V0`/`V1`: observed stable pattern; versioned fresh assignment and writer acceptance remain unproved. |
| `charge_raw` | PROVIDER_GENERATED | Fresh-modern charge `-50..=50` maps to byte `charge + 50`, copied within each group. The observed central states support the offset; wider admitted values are user-selected profile policy. Legacy translation retains a separate blocker. |
| `entry_number` | UNRESOLVED_BLOCKER | `V0`/`V1`: same group start and consecutive ID-1/ID-7/CP stream-order progression. Two fresh singleton groups independently restart the sequence; larger groups and rollover remain outside evidence. |
| `reference[2]` | UNRESOLVED_BLOCKER | `V0`: group and CP slots remain zero when reference metadata is added, while phase slots change. No-reference pattern is supported; active input-to-slot mapping remains unproved. |
| `timestamp_ole` | UNRESOLVED_BLOCKER | `V0`: ID-1/ID-7/CP timestamps agree within each group, and two groups in one file have distinct timestamps. ID-9 date stays fixed. `V1`: CP copies ID-1, but ID-7 can differ. Fresh clock source/rounding unknown. |
| `unknown[2]` | UNRESOLVED_BLOCKER | `V0`/`V1`: stable by record kind and matching between profiles; versioned assignment and writer acceptance remain unproved. |

These slots are `RAW_PRESERVED_ONLY` when editing existing databases, but
`UNRESOLVED_BLOCKER` for fresh records. The plan retains source provenance and
G encounter identity semantically; it does not smuggle any of these raw slots
through caller input.

## ID-1 formula/compound group

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| chunk ID, group position | PROVIDER_GENERATED | `CODE`: ID 1 and database → group → phases → ranges → comments ordering. |
| shared header | UNRESOLVED_BLOCKER | See preceding table. |
| `compound_name` | UNRESOLVED_BLOCKER | `V0`: empty in both user-declared neutral/charged groups, while their formula labels differ and the local function name is reused. `V1`: compound name may serve as a separate alias. Fresh omitted/supplied policy requires explicit scope. |
| `formula_name` | CALLER_PROVIDED | `CODE`/`CONTRACT`: target formula **label**, max 40 printable ASCII bytes. Full semantic identity is exact elemental ratio plus charge; the label is neither parsed for charge nor used as the sole key. |
| `real_stoichiometric_coefficients[7]` | UNRESOLVED_BLOCKER | `V0`: stable through edits and identical across the neutral/charged groups. `V1`: simple integer labels matched written amounts; fresh input-to-slot scale and order remain unproved. |
| `unit_energy` | CALLER_PROVIDED | `CODE`: codes 0 calorie and 1 joule; paired Legacy/FDB samples use 1 (`PAIR`). No implicit conversion in the plan. |
| `unit_pressure` | CALLER_PROVIDED | `CODE`: codes 0 atmosphere and 1 bar; the source/provider must choose a known convention. This does not imply a reference pressure. |
| `reserved_string_1`, `reserved_string_2` | UNRESOLVED_BLOCKER | `V0`: stable and nonzero across edits; assignment policy and version scope unknown. |
| `unknown[4]`, `padding_final[24]` | UNRESOLVED_BLOCKER | `V0`/`V1`: zero in observed FDBs; provider-written acceptance remains unproved. |

Database-global grouping is by full composition-plus-charge identity, with one
ordered ID-1 group holding many independently named functions. Identical
ratios at the same charge in two groups are rejected; identical ratios at
different charges are allowed. Numerically equal functions remain distinct.

## ID-7 ordinary/base function

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| chunk ID, placement | PROVIDER_GENERATED | `CODE`: ID 7; phase precedes its ranges inside one group. |
| shared header | UNRESOLVED_BLOCKER | See shared-entry table. |
| `phase_name` | CALLER_PROVIDED | `V0`: direct-modern function name is a nonempty caller/native-modern name. `V1`: matched translations use `<FILE phase ID>_<zero-based four digits>[A]`. These are separate naming policies. |
| `enthalpy`, `entropy` | CALLER_PROVIDED | `CODE`: independent ID-7 H/S values. The plan retains them on both ordinary and nonzero A functions; C3 can write supplied values without choosing an anchor. Legacy source-to-plan selection remains a later scientific integration question. |
| `phase_id_raw` | UNRESOLVED_BLOCKER | `V0`: fresh solid singleton groups reuse the same phase ID, including a declared neutral/charged pair; negatives and CP links are group-local. `V1`: multiple solid functions are consecutive within groups. A second fresh function inside one group and other states remain unproved. |
| `phase_id_raw_neg` | PROVIDER_GENERATED | `LOCAL_PAIR`: arithmetic negation of positive ID holds in the paired FDB and comparison CDBs; no independent blocker after positive ID allocation. |
| `density_raw`, thermal-expansion, compressibility, bulk-modulus, magnetic temperature/moment, `p_factor` | EVIDENCED_DEFAULT | `CODE`: exact numeric zero is the inactive pure H/S/Cp pattern. Any requested active contribution is a typed semantic blocker; zero is only a profile-specific inactive value, not a default for general FDBs. |
| `padding_1`, `padding_2` | UNRESOLVED_BLOCKER | `V0`/`V1`: zero in observed phases, pending versioned writer acceptance. |

ID-8 transition phase is `NOT_REQUIRED_FOR_PROFILE`; a requested active
transition is rejected. A source with active magnetic/pressure-volume physics
cannot enter this pure profile by silently selecting zero.

## CP ranges under ordinary and added functions

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| chunk ID | UNRESOLVED_BLOCKER | `V0`: every fresh nonzero-Cp range uses ID-2, including all three ranges under one phase. `V1`: ID-2 has nonzero Cp and ID-5 exactly zero Cp under base and A names. ID-2 is supported for bounded fresh nonzero Cp; fresh zero-Cp kind or canonical policy remains open. Wider IDs 3/4/6 are outside this profile. |
| shared header | UNRESOLVED_BLOCKER | See shared-entry table. |
| `phase_id_raw` | PROVIDER_GENERATED | `CODE`: exact link to owning ID-7, conditional on resolving the native ID allocation rule. |
| `temperature_min`, `temperature_max` | CALLER_PROVIDED | `CODE`: finite, positive, ordered, contiguous kelvin bounds. `LOCAL_PAIR`: identity-linked base objects preserve counted source-range cardinality and upper bounds in encounter order across represented models. First lower-bound and added-range rules remain unresolved. |
| `enthalpy`, `entropy` | CALLER_PROVIDED | `CODE`: range-specific H/S constants at 298.15 K in group energy units. `PAIR`: direct conversion holds for a tested small-fixture subset, but full local base ranges include transformed H/S anchors. The full local ID-5 A translation has opposite-sign nonzero S, unlike earlier small examples. Source-to-plan conversion is model/function dependent. |
| `coefficients[8]`, `powers[8]` | CALLER_PROVIDED | `CODE`: eight ordered slots. `V0`: last coefficient and power slots are zero, but some zero-coefficient slots have nonzero powers, so blanket zero-filling is invalid. `V1`: tested small fixtures map four fixed and three variable Legacy terms into slots 0–6; full source conversion varies. Preserve exact supplied terms; shorter-list fill rule remains `UNRESOLVED_BLOCKER`. |
| `unknown_1[4]`, `padding_remaining[56]` | UNRESOLVED_BLOCKER | `V0`/`V1`: zero in observed fresh ID-2 and translated ID-2/ID-5 records; versioned writer acceptance remains open. |

The rigorous **translation** plan always retains a base and A identity, even
for zero base and zero A objects. The directly authored empty function has an
ID-7 with no CP and no A companion; it does not establish the translation
omission policy. `ExplicitZeroOrdinary` and `ExplicitZero` have no fabricated
Cp interval: local translations emit some zero objects and omit others, so
translated zero-block encoding or a versioned omission policy is unresolved. A
nonzero A may carry explicit ordered H/S/Cp interval intent, but its Legacy
leading-pair → CP bounds, powers, phase anchors, and exceptional entropy-sign
behavior are not yet a general rule.

The fresh empty function needs a structural verification path: the ordinary
thermodynamic view deliberately rejects a phase with no CP ranges. This is
engineering work, separate from the observed native empty-function form.

ID-10 comments and ID-11 extended-property records are
`NOT_REQUIRED_FOR_PROFILE`. Existing records remain `RAW_PRESERVED_ONLY` in the
parser/editor. Active ID-11 data is a typed blocker for fresh pure H/S/Cp plans.

## Exact evidence needed for FDB-C3

The table below is the C3 field disposition checklist. `RESOLVED` means the
semantic/caller value and native location are established for this profile;
it does not mean the builder has been written. `ENGINEERING_ONLY` means a
provider-owned implementation can proceed using existing code. `EVIDENCE_REQUIRED`
and `SCIENTIFIC_SEMANTICS_REQUIRED` forbid a guessed native value. For exact
controlled experiments and dependencies, see
[`fdb-c3-readiness.md`](fdb-c3-readiness.md).

| Field/capability | C3 disposition | Current evidence and exact remaining action | User/domain input? | Can native construction proceed without it? |
| --- | --- | --- | --- | --- |
| ID-9 chunk ID, `CMPD`, comment, caller OLE date | RESOLVED | Parser's required magic, fixed text and date validator; write supplied date/comment. | No | Yes |
| ID-9 `read_flag` | EVIDENCE_REQUIRED | Fresh and translated FDBs agree on zero; named-version initialization and provider-written acceptance remain. | Yes, versioned acceptance | No |
| ID-9 padding 1/2/3 and unknown 1/2 | EVIDENCE_REQUIRED | Fresh padding is zero; both unknown regions are stable through edits, but the second differs from translation. Need versioned assignment and writer acceptance. | Yes, versioned fresh input | No |
| Shared `element_ids[7]`, integer `element_coefficients[7]` | EVIDENCE_REQUIRED | Fresh group-wide copying and equality across neutral/charged groups are confirmed; input-linked code/order/scale remain. | Yes, recorded stoichiometry inputs | No |
| Shared coefficient padding and unknown 2 bytes | EVIDENCE_REQUIRED | Stable by kind in fresh edits and matching translated counterparts; versioned writer assignment remains. | Yes | No |
| Shared `charge_raw` | RESOLVED | Fresh-modern profile uses the checked `charge + 50` byte rule for `-50..=50` and repeats it on ID-1/ID-7/CP. Legacy translation is separate. | No for fresh profile | Yes for bounded fresh profile |
| Shared `entry_number` | RESOLVED | Fresh and translated FDBs share group start and consecutive stream-order increments within the observed small profile; rollover remains outside it. | No for one-function profile | Yes for bounded small profile |
| Shared `reference[2]` | EVIDENCE_REQUIRED | No-reference fresh group/CP slots are zero; active reference edit changes only phase slots. Input-to-slot assignment remains. | Yes for active references | No for active; bounded no-reference policy possible |
| Shared `timestamp_ole` | POLICY_REQUIRED | Fresh ID-1/7/CP agree per group; two groups in one file differ while ID-9 date stays fixed. Clock source/rounding remain unproved. | Yes, versioned action times | No |
| ID-1 formula label bytes and energy/pressure codes | RESOLVED | Preserve the caller-supplied formula spelling exactly after `chemformula` validation; reject invalid formulas rather than canonicalizing/repairing them. Element order is retained exactly as entered. Known code enums; pressure reference is not inferred. | No | Yes |
| ID-1 formula label correspondence to semantic composition/charge | EVIDENCE_REQUIRED | Neutral/charged groups differ in formula label and raw charge but share native element arrays. Label spelling is not authority for signed charge or composition. Record exact UI inputs and alias policy. | Yes | No |
| ID-1 `compound_name`, real coefficients, reserved strings, unknown/padding | EVIDENCE_REQUIRED | Fresh compound name stays empty; real coefficients and reserved fields are unchanged across charge contrast. Input-linked scale and versioned assignment remain. | Yes, recorded stoichiometry/version | No |
| ID-7 target phase name and explicit H/S | RESOLVED | Fresh names are caller supplied; translated names derive from FILE phase ID/G encounter index. The plan keeps explicit H/S. | No | Yes |
| ID-7 `phase_id_raw` allocation and physical padding | EVIDENCE_REQUIRED | Two fresh singleton groups reuse a solid phase ID with correct negatives/links. Allocation inside one group and versioned padding acceptance remain. | Yes, same-group pair if admitted | No |
| ID-7 `phase_id_raw_neg` relation | RESOLVED | Compute arithmetic negative after allocating the positive ID; observed in paired FDB and comparison CDBs. | No | Yes, conditional on positive ID |
| ID-7 inactive magnetic/pressure-volume tail for pure H/S/Cp | RESOLVED | Provider eligibility code accepts exact numeric zero; active intent fails plan validation. | No | Yes for admitted profile |
| CP ID-2/ID-5 kind for counted base ranges | EVIDENCE_REQUIRED | Fresh nonzero Cp uses ID-2; translated counted bases split ID-2 nonzero / ID-5 zero. Fresh zero-Cp selector or accepted canonical policy remains. | Fresh zero-Cp output or acceptance | Yes for bounded fresh nonzero Cp |
| CP ID-2/ID-5 kind for added/generated ranges | SCIENTIFIC_SEMANTICS_REQUIRED | The target zero/nonzero split also holds under A names, but source leading terms do not map directly to target Cp. Derive the source reduction and confirm fresh selection. | Controlled source/output | No |
| CP supplied bounds/anchors/terms and phase-ID link | ENGINEERING_ONLY | Shared CP layout and group-local exact phase link known; plan retains explicit values. Base counted-range cardinality and upper bounds transfer in order. Source-to-plan H/S/Cp transforms are a separate scientific blocker. | No for explicit plan | Yes, conditional on kind and ID rules |
| CP unknown 4 bytes and remaining padding | EVIDENCE_REQUIRED | Zero in fresh ID-2 and translated ID-2/5; versioned writer acceptance remains. | Yes | No |
| CP unused coefficient/power slots for fewer than seven terms | EVIDENCE_REQUIRED | Paired seven-term sources have zero slot 8; shorter lists need controlled native fill evidence. | Yes if admitted | No for shorter profile |
| Explicit zero base/A physical form | EVIDENCE_REQUIRED | Fresh empty function has ID-7 without CP and no A; translated zero blocks may be emitted or omitted. Keep profile policies separate. | Yes for translation policy | Fresh empty form observed; translation blocked |
| Fresh empty-function verification | ENGINEERING_ONLY | Strict reparse and domain validation can check the ID-7-only structure; ordinary H/S/Cp evaluation requires a CP range and must be skipped for this form. | No | Yes after builder implementation |
| General Legacy ordinary H/S/Cp source mapping | SCIENTIFIC_SEMANTICS_REQUIRED | Full local base ranges preserve cardinality and upper bounds but show direct and transformed H/S/Cp values by model/function; the plan accepts explicit target values. | Yes if conversion adapter is built | C3 builder yes; full conversion no |
| Legacy A leading pair → CP and entropy sign | SCIENTIFIC_SEMANTICS_REQUIRED | In this full translation, emitted ID-5 A H is direct and nonzero S has opposite sign; earlier small paired examples mostly had direct S. ID-2 A source mapping also differs. Determine the governing source/version condition. | Yes if adapter is built | C3 builder yes; full conversion no |
| Exact conversion of arbitrary source `f64` composition to rational identity | POLICY_REQUIRED | No tolerance is selected. An adapter must preserve source exact fractions or declare a conversion policy. | Domain input may be needed | C3 with exact-rational input yes |
| NativeFactSage omission choice for zero objects | POLICY_REQUIRED | Only after version behavior is observed; do not silently omit in rigorous plan. | Domain/version policy later | No for zero-object native output |
| Provider raw constructor and serialize/reparse/domain/thermo pipeline | ENGINEERING_ONLY | Existing lower layers are available; implement after native field gates. | No | Yes once evidence gates close |
| Independent FactSage acceptance of newly built file | EVIDENCE_REQUIRED | Verification-class gate: self-reparse is insufficient; open a tiny constructed FDB in the identified FactSage version. | Yes, controlled local check | Construction can be coded; GO publication cannot |

ID-8 transitions and active ID-11/auxiliary physics are
`NOT_REQUIRED_FOR_ADMITTED_PROFILE`. They fail closed rather than acquiring
fabricated zero defaults. No native field is silently initialized by Rust's
default values. Historical `dbsolution` has Legacy/SLN charge parsing but no
FDB raw constructor or evidence for the unresolved shared-header fields.


## Bounded fresh-modern policy overrides

The detailed inventory above records the evidence history. For the first
`FreshModern` writer, the normative construction policy is now
[`fdb-bounded-fresh-profile.md`](fdb-bounded-fresh-profile.md). Where an older
inventory row still says `UNRESOLVED_BLOCKER` or `EVIDENCE_REQUIRED`, the
bounded profile takes precedence when it explicitly supplies a policy or excludes
the feature.

In particular:

| Field/capability | Bounded fresh-modern disposition |
| --- | --- |
| ID-9 opaque/padding/read-flag bytes | Copy from the controlled empty-FDB template; change only established date/comment fields. |
| ID-9 / entry timestamps | OLE Automation `f64`; explicit build timestamp or provider clock policy. |
| Formula parser | Use `chemformula`. |
| Element IDs | Periodic-table atomic numbers encoded as `u8`. |
| Element slot order | **CONFIRMED DOMAIN POLICY:** preserve first-appearance/parser encounter order exactly; verify against the existing local corpus as a regression check. |
| Integer stoichiometry | Parsed integral formula coefficients. |
| Real stoichiometry | **CONFIRMED NATIVE TYPE:** `f8[7]` / `[f64; 7]`. Default to the same formula coefficients as `f64`; advanced real-stoichiometry overrides are outside the first profile. |
| Charge | `raw = semantic + 50` for semantic `-50..=50`. |
| Compound name | Empty for the bounded fresh profile, matching the direct-modern evidence. |
| Function name | Caller supplied; no Legacy naming rule. |
| References/density | Inactive/zero only in the first profile. |
| Function topology | One ordinary solid function per formula group. |
| Cp kind | Nonzero Cp only; ID-2. Fresh zero-Cp is outside the first profile. |
| Per-record opaque padding/defaults | Use the controlled fresh record-kind values and validate by provider-built FactSage acceptance. |

No item in this bounded table requires a new human/domain decision. Remaining
uncertainty must be handled by automated corpus checks, a narrower capability
gate, or implementation/verification failure reporting.


Formula-label policy: **CONFIRMED DOMAIN POLICY**. Preserve caller spelling exactly
after `chemformula` validation; reject invalid formulas and require corrected input.
Do not canonicalize or reorder equivalent formulas.