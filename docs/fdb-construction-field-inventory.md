# Fresh FDB field inventory (FDB-C1)

This inventory covers the rigorous pure H/S/Cp Legacy → Modern Function profile.
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
| Every non-database raw record has `RawCommonHeader.charge_raw: i8`; parser and serializer preserve it. | CONFIRMED_BY_PROVIDER_CODE | There is no semantic charge accessor on `CompoundView`; a fresh writing rule is not established. |
| Formula label and composition are separately stored in ID-1; shared headers occur on ID-1, ID-7 and CP records. | CONFIRMED_BY_PROVIDER_CODE_AND_LOCAL_PAIR | The paired FDB repeats its raw charge within each group; this does not prove semantic charge encoding or a fresh constructor rule. |
| `Fe3O4` and `Fe3O4[+]` must remain distinct groups. | CONFIRMED_BY_DOMAIN_CONTRACT | Label syntax is not parsed as semantic charge. |
| Solution parser models source charge as `i32` and keeps bracketed charge spelling in external references. | CONFIRMED_BY_PROVIDER_CODE | Its SLN parser test is synthetic, not paired FDB evidence. |
| Historical `dbsolution` parses Legacy charge and derives SLN endmember charge from `chemformula`. | HISTORICAL_PYTHON_HINT | This does not prove FDB `charge_raw` initialization. |
| Numeric `charge_raw` mapping in a controlled neutral/charged FDB pair. | UNKNOWN | The local pair has differing raw charges but no neutral group; unmarked formula labels can have a nonzero raw charge. |

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

## ID-9 database header

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| chunk ID | PROVIDER_GENERATED | `CODE`: ID 9 and first-record ordering are fixed. |
| `magic` | EVIDENCED_DEFAULT | `CODE`: parser requires `CMPD`. |
| `date_ole` | CALLER_PROVIDED | `CODE`: OLE date field; plan uses the existing representable-date validator. Whether native writer should use a particular clock source is a separate policy. |
| `read_flag` | UNRESOLVED_BLOCKER | `CODE`: zero is an observed FDB-compatible guardrail, but also occurs in CDBs. A controlled fresh FDB must confirm the writing rule under a named version. |
| `comment` | CALLER_PROVIDED | `CODE`: 80-byte fixed ASCII; empty is explicit and allowed. |
| `padding_1`, `padding_2`, `padding_3` | UNRESOLVED_BLOCKER | `UNKNOWN`: exact fresh-output values/padding convention need a paired native creation experiment. |
| `unknown_1`, `unknown_2` | UNRESOLVED_BLOCKER | `UNKNOWN`: meaning and fresh-output values are not established. |

There is no native database name/identifier field in this header. A filename or
logical bundle role belongs to orchestration (`NOT_REQUIRED_FOR_PROFILE` as a
header field). No invented database ID is in the semantic plan.

## Shared entry header (ID-1, ID-7, ID-2 and ID-5)

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| `element_ids[7]` | UNRESOLVED_BLOCKER | `LOCAL_PAIR`: strict simple integer FDB labels match atomic-number IDs in label order, with unused slots zero and group-wide repetition. Fractional, charged and pseudocomponent encoding and fresh construction remain unproved. |
| `element_coefficients[7]` | UNRESOLVED_BLOCKER | `LOCAL_PAIR`: the same bounded labels have integer coefficients matching the written amounts and group-wide repetition. General native scale and fresh construction remain unproved. |
| `coefficient_padding` | UNRESOLVED_BLOCKER | `UNKNOWN`: no fresh default. |
| `charge_raw` | UNRESOLVED_BLOCKER | `LOCAL_PAIR`: the raw signed byte repeats within each FDB group and differs among groups. The corpus lacks a neutral group and does not establish numeric semantic mapping. |
| `entry_number` | UNRESOLVED_BLOCKER | `LOCAL_PAIR`: consecutive in ID-1/ID-7/CP stream order within every paired FDB group, from a uniform group start. Comparison CDB groups do not all follow this pattern; fresh start/rollover remain unproved. |
| `reference[2]` | UNRESOLVED_BLOCKER | `LOCAL_PAIR`: both slots are zero in the paired FDB, while comparison CDBs contain nonzero values. CP references match their owning phase; fresh generation remains unproved. |
| `timestamp_ole` | UNRESOLVED_BLOCKER | `LOCAL_PAIR`: CP timestamps copy ID-1 group timestamps in the paired FDB and both comparison CDBs. ID-7 may differ from ID-1 and CP; fresh ID-1/ID-7 source unknown. |
| `unknown[2]` | UNRESOLVED_BLOCKER | `UNKNOWN`: no established fresh default. |

These slots are `RAW_PRESERVED_ONLY` when editing existing databases, but
`UNRESOLVED_BLOCKER` for fresh records. The plan retains source provenance and
G encounter identity semantically; it does not smuggle any of these raw slots
through caller input.

## ID-1 formula/compound group

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| chunk ID, group position | PROVIDER_GENERATED | `CODE`: ID 1 and database → group → phases → ranges → comments ordering. |
| shared header | UNRESOLVED_BLOCKER | See preceding table. |
| `compound_name` | UNRESOLVED_BLOCKER | `LOCAL_PAIR`: 40-byte text can differ from formula text, and Modern references can use either; fresh selection/alias rule unknown. |
| `formula_name` | CALLER_PROVIDED | `CODE`/`CONTRACT`: target formula **label**, max 40 printable ASCII bytes. Full semantic identity is exact elemental ratio plus charge; the label is neither parsed for charge nor used as the sole key. |
| `real_stoichiometric_coefficients[7]` | UNRESOLVED_BLOCKER | `LOCAL_PAIR`: strict simple integer FDB labels match written amounts in label order with empty slots zero. Fractional/charged/pseudocomponent encoding and fresh scale remain unproved. |
| `unit_energy` | CALLER_PROVIDED | `CODE`: codes 0 calorie and 1 joule; paired Legacy/FDB samples use 1 (`PAIR`). No implicit conversion in the plan. |
| `unit_pressure` | CALLER_PROVIDED | `CODE`: codes 0 atmosphere and 1 bar; the source/provider must choose a known convention. This does not imply a reference pressure. |
| `reserved_string_1`, `reserved_string_2` | UNRESOLVED_BLOCKER | `UNKNOWN`: fresh contents and whether truly blank are not established. |
| `unknown[4]`, `padding_final[24]` | UNRESOLVED_BLOCKER | `UNKNOWN`: no evidenced native default. |

Database-global grouping is by full composition-plus-charge identity, with one
ordered ID-1 group holding many independently named functions. Identical
ratios at the same charge in two groups are rejected; identical ratios at
different charges are allowed. Numerically equal functions remain distinct.

## ID-7 ordinary/base function

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| chunk ID, placement | PROVIDER_GENERATED | `CODE`: ID 7; phase precedes its ranges inside one group. |
| shared header | UNRESOLVED_BLOCKER | See shared-entry table. |
| `phase_name` | CALLER_PROVIDED | `PAIR`/`CONTRACT`: `<FILE phase ID>_<zero-based four digits>`, 40-byte text; display name is not the naming source. |
| `enthalpy`, `entropy` | CALLER_PROVIDED | `CODE`: independent ID-7 H/S values. The plan retains them on both ordinary and nonzero A functions; C3 can write supplied values without choosing an anchor. Legacy source-to-plan selection remains a later scientific integration question. |
| `phase_id_raw` | UNRESOLVED_BLOCKER | `LOCAL_PAIR`: all observed FDB phases are solid; IDs are consecutive in group stream order, restart in each group, and CP links group-locally. Fresh allocation and other phase states remain unproved. |
| `phase_id_raw_neg` | PROVIDER_GENERATED | `LOCAL_PAIR`: arithmetic negation of positive ID holds in the paired FDB and comparison CDBs; no independent blocker after positive ID allocation. |
| `density_raw`, thermal-expansion, compressibility, bulk-modulus, magnetic temperature/moment, `p_factor` | EVIDENCED_DEFAULT | `CODE`: exact numeric zero is the inactive pure H/S/Cp pattern. Any requested active contribution is a typed semantic blocker; zero is only a profile-specific inactive value, not a default for general FDBs. |
| `padding_1`, `padding_2` | UNRESOLVED_BLOCKER | `UNKNOWN`: no established fresh byte pattern. |

ID-8 transition phase is `NOT_REQUIRED_FOR_PROFILE`; a requested active
transition is rejected. A source with active magnetic/pressure-volume physics
cannot enter this pure profile by silently selecting zero.

## CP ranges under ordinary and added functions

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| chunk ID | UNRESOLVED_BLOCKER | `LOCAL_PAIR`: both ID 2 and ID 5 appear under base and A names. In this FDB, ID-2 always has nonzero Cp and ID-5 always has exactly zero Cp; every identity-and-order-linked counted base range follows the same source zero/nonzero split. Whether that property causes kind selection for fresh or generated records remains unproved. Wider ID 3/4/6 families are outside this profile. |
| shared header | UNRESOLVED_BLOCKER | See shared-entry table. |
| `phase_id_raw` | PROVIDER_GENERATED | `CODE`: exact link to owning ID-7, conditional on resolving the native ID allocation rule. |
| `temperature_min`, `temperature_max` | CALLER_PROVIDED | `CODE`: finite, positive, ordered, contiguous kelvin bounds. `LOCAL_PAIR`: identity-linked base objects preserve counted source-range cardinality and upper bounds in encounter order across represented models. First lower-bound and added-range rules remain unresolved. |
| `enthalpy`, `entropy` | CALLER_PROVIDED | `CODE`: range-specific H/S constants at 298.15 K in group energy units. `PAIR`: direct conversion holds for a tested small-fixture subset, but full local base ranges include transformed H/S anchors. The full local ID-5 A translation has opposite-sign nonzero S, unlike earlier small examples. Source-to-plan conversion is model/function dependent. |
| `coefficients[8]`, `powers[8]` | CALLER_PROVIDED | `CODE`: capacity of eight ordered Cp terms; `PAIR`: four fixed and three variable Legacy terms map to slots 0–6 for a tested small-fixture subset. The full local base translation includes both direct and transformed term mappings. The eighth slot is zero in observed CP records, without proving a fresh fill rule. The semantic plan takes ordered meaningful terms without native slot padding. Exact caller-supplied values/powers are retained; no refit, normalization, or tiny-term erasure. The native fill rule for unused slots remains `UNRESOLVED_BLOCKER` outside the paired seven-term profile. |
| `unknown_1[4]`, `padding_remaining[56]` | UNRESOLVED_BLOCKER | `UNKNOWN`: no fresh-output default. |

The rigorous plan always retains a base and A identity, even for zero base and
zero A objects. `ExplicitZeroOrdinary` and `ExplicitZero` have no fabricated
Cp interval: local translations emit some zero objects and omit others, so
native zero-block encoding or a versioned omission policy is unresolved. A
nonzero A may carry explicit ordered H/S/Cp interval intent, but its Legacy
leading-pair → CP bounds, powers, phase anchors, and exceptional entropy-sign
behavior are not yet a general rule.

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
| ID-9 `read_flag` | EVIDENCE_REQUIRED | Zero is compatible with examined FDBs but not a fresh-construction rule; confirm with a controlled fresh FDB under a named version. | Yes, tiny local fixture | No |
| ID-9 padding 1/2/3 and unknown 1/2 | EVIDENCE_REQUIRED | Parser preserves bytes only; compare controlled fresh FDB headers under fixed version. | Yes, tiny local fixture | No |
| Shared `element_ids[7]`, integer `element_coefficients[7]` | EVIDENCE_REQUIRED | Arrays parsed; need element code/order, scale, unused slots and repetition in ID-1/7/CP. | Yes, neutral/fractional paired fixture | No |
| Shared coefficient padding and unknown 2 bytes | EVIDENCE_REQUIRED | Distinct preserved slots; inspect each record kind in one fresh FDB. | Yes | No |
| Shared `charge_raw` | EVIDENCE_REQUIRED | Signed `i8` parsed in each record; need direct semantic charge mapping, neutral zero and per-record repetition. | Yes, neutral/charged pair | No |
| Shared `entry_number` | EVIDENCE_REQUIRED | Paired FDB uses a consecutive group-stream counter; confirm fresh start, increments and rollover. | Yes | No |
| Shared `reference[2]` | EVIDENCE_REQUIRED | Paired FDB values are zero, comparison CDBs include nonzero values, and CP copies its phase; fresh FDB source/initialization unknown. | Yes | No |
| Shared `timestamp_ole` | EVIDENCE_REQUIRED | CP copies ID-1 in all local native members; fresh ID-1 and ID-7 timestamp sources remain unknown. | Yes | No |
| ID-1 formula label bytes and energy/pressure codes | RESOLVED | Supplied printable label; known code enums; pressure reference is not inferred. | No | Yes |
| ID-1 formula label correspondence to semantic composition/charge | EVIDENCE_REQUIRED | The label is opaque; establish provider-accepted grammar and consistency rule with a controlled neutral/charged pair. | Yes | No |
| ID-1 `compound_name`, real coefficients, reserved strings, unknown/padding | EVIDENCE_REQUIRED | Native locations parsed; need name selection, coefficient scaling and fresh reserved patterns. | Yes, changed-label/fractional/charged examples | No |
| ID-7 target phase name and explicit H/S | RESOLVED | Name derives from FILE phase ID/G encounter index; plan supplies both H/S fields. | No | Yes |
| ID-7 `phase_id_raw` allocation and physical padding | EVIDENCE_REQUIRED | Group-local consecutive solid IDs and arithmetic negative IDs observed; fresh allocation, other states and padding need controlled creation. | Yes, two base/A pairs | No |
| ID-7 `phase_id_raw_neg` relation | RESOLVED | Compute arithmetic negative after allocating the positive ID; observed in paired FDB and comparison CDBs. | No | Yes, conditional on positive ID |
| ID-7 inactive magnetic/pressure-volume tail for pure H/S/Cp | RESOLVED | Provider eligibility code accepts exact numeric zero; active intent fails plan validation. | No | Yes for admitted profile |
| CP ID-2/ID-5 kind for counted base ranges | EVIDENCE_REQUIRED | Identity-and-order-linked source ranges select ID-2 for nonzero Cp and ID-5 for exact-zero Cp across represented models; confirm as a fresh writer rule. | Controlled source/output | No |
| CP ID-2/ID-5 kind for added/generated ranges | SCIENTIFIC_SEMANTICS_REQUIRED | The target zero/nonzero split also holds under A names, but source leading terms do not map directly to target Cp. Derive the source reduction and confirm fresh selection. | Controlled source/output | No |
| CP supplied bounds/anchors/terms and phase-ID link | ENGINEERING_ONLY | Shared CP layout and group-local exact phase link known; plan retains explicit values. Base counted-range cardinality and upper bounds transfer in order. Source-to-plan H/S/Cp transforms are a separate scientific blocker. | No for explicit plan | Yes, conditional on kind and ID rules |
| CP unknown 4 bytes and remaining padding | EVIDENCE_REQUIRED | Parsed/preserved; need fresh ID-2 and ID-5 bytes. | Yes | No |
| CP unused coefficient/power slots for fewer than seven terms | EVIDENCE_REQUIRED | Paired seven-term sources have zero slot 8; shorter lists need controlled native fill evidence. | Yes if admitted | No for shorter profile |
| Explicit zero base/A physical form | EVIDENCE_REQUIRED | Paired exports sometimes omit zero blocks; rigorous identities retained. Need accepted explicit records or versioned omission behavior. | Yes, zero base/A pair | No for zero objects |
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
