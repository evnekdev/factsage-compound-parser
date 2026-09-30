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

## ID-9 database header

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| chunk ID | PROVIDER_GENERATED | `CODE`: ID 9 and first-record ordering are fixed. |
| `magic` | EVIDENCED_DEFAULT | `CODE`: parser requires `CMPD`. |
| `date_ole` | CALLER_PROVIDED | `CODE`: OLE date field; plan uses the existing representable-date validator. Whether native writer should use a particular clock source is a separate policy. |
| `read_flag` | EVIDENCED_DEFAULT | `CODE`: zero is the observed FDB-compatible guardrail, not an intrinsic FDB classifier. Fresh FDB materialization must use logical caller role plus this guardrail. |
| `comment` | CALLER_PROVIDED | `CODE`: 80-byte fixed ASCII; empty is explicit and allowed. |
| `padding_1`, `padding_2`, `padding_3` | UNRESOLVED_BLOCKER | `UNKNOWN`: exact fresh-output values/padding convention need a paired native creation experiment. |
| `unknown_1`, `unknown_2` | UNRESOLVED_BLOCKER | `UNKNOWN`: meaning and fresh-output values are not established. |

There is no native database name/identifier field in this header. A filename or
logical bundle role belongs to orchestration (`NOT_REQUIRED_FOR_PROFILE` as a
header field). No invented database ID is in the semantic plan.

## Shared entry header (ID-1, ID-7, ID-2 and ID-5)

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| `element_ids[7]` | UNRESOLVED_BLOCKER | `CODE` proves storage; element-symbol → native ID assignment, unused slots and group/phase/range repetition need paired construction evidence. |
| `element_coefficients[7]` | UNRESOLVED_BLOCKER | Caller supplies positive stoichiometry, but encoding/rounding and repetition into each record are unproved. |
| `coefficient_padding` | UNRESOLVED_BLOCKER | `UNKNOWN`: no fresh default. |
| `charge_raw` | UNRESOLVED_BLOCKER | `CODE` preserves signed byte; charged-formula policy and default for neutral groups need evidence. |
| `entry_number` | UNRESOLVED_BLOCKER | `UNKNOWN`: source entry number is preserved but generation/relationship semantics are not established. |
| `reference[2]` | UNRESOLVED_BLOCKER | `UNKNOWN`: native reference semantics and fresh values are not established. |
| `timestamp_ole` | UNRESOLVED_BLOCKER | `CODE` identifies OLE field; whether it copies database date, creation time, or source time is unknown. |
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
| `compound_name` | UNRESOLVED_BLOCKER | `CODE`: 40-byte text, but fresh FDB distinction between compound name and formula text is not fully established. |
| `formula_name` | CALLER_PROVIDED | `CODE`/`CONTRACT`: explicit formula-group identity, max 40 printable ASCII bytes in the plan. |
| `real_stoichiometric_coefficients[7]` | UNRESOLVED_BLOCKER | Caller supplies composition; native slot mapping and when real rather than integer coefficients are used require evidence. |
| `unit_energy` | CALLER_PROVIDED | `CODE`: codes 0 calorie and 1 joule; paired Legacy/FDB samples use 1 (`PAIR`). No implicit conversion in the plan. |
| `unit_pressure` | CALLER_PROVIDED | `CODE`: codes 0 atmosphere and 1 bar; the source/provider must choose a known convention. This does not imply a reference pressure. |
| `reserved_string_1`, `reserved_string_2` | UNRESOLVED_BLOCKER | `UNKNOWN`: fresh contents and whether truly blank are not established. |
| `unknown[4]`, `padding_final[24]` | UNRESOLVED_BLOCKER | `UNKNOWN`: no evidenced native default. |

Database-global grouping is by composition, with one ordered ID-1 group holding
many independently named functions. Identical stoichiometry in two ID-1 groups
is rejected by the semantic plan. Numerically equal functions remain distinct.

## ID-7 ordinary/base function

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| chunk ID, placement | PROVIDER_GENERATED | `CODE`: ID 7; phase precedes its ranges inside one group. |
| shared header | UNRESOLVED_BLOCKER | See shared-entry table. |
| `phase_name` | CALLER_PROVIDED | `PAIR`/`CONTRACT`: `<FILE phase ID>_<zero-based four digits>`, 40-byte text; display name is not the naming source. |
| `enthalpy`, `entropy` | CALLER_PROVIDED | `CODE`: independent ID-7 H/S values. `PAIR` has a selected 298.15 K anchor, but the general selection rule versus each CP range remains unresolved for C3. The plan retains both. |
| `phase_id_raw` | UNRESOLVED_BLOCKER | `CODE`: CP links by exact ID within group; fresh allocation/state/uniqueness rule not proved. |
| `phase_id_raw_neg` | UNRESOLVED_BLOCKER | `UNKNOWN`: relationship to positive ID is not established. |
| `density_raw`, thermal-expansion, compressibility, bulk-modulus, magnetic temperature/moment, `p_factor` | EVIDENCED_DEFAULT | `CODE`: exact numeric zero is the inactive pure H/S/Cp pattern. Any requested active contribution is a typed semantic blocker; zero is only a profile-specific inactive value, not a default for general FDBs. |
| `padding_1`, `padding_2` | UNRESOLVED_BLOCKER | `UNKNOWN`: no established fresh byte pattern. |

ID-8 transition phase is `NOT_REQUIRED_FOR_PROFILE`; a requested active
transition is rejected. A source with active magnetic/pressure-volume physics
cannot enter this pure profile by silently selecting zero.

## Ordinary ID-2 Cp ranges and added ID-5 companion

| Native field | Class | Evidence and required disposition |
| --- | --- | --- |
| chunk ID | PROVIDER_GENERATED | `PAIR`: counted Legacy ordinary intervals use ID 2; emitted A companions use ID 5. Wider ID 3/4/6 families are `NOT_REQUIRED_FOR_PROFILE`. |
| shared header | UNRESOLVED_BLOCKER | See shared-entry table. |
| `phase_id_raw` | PROVIDER_GENERATED | `CODE`: exact link to owning ID-7, conditional on resolving the native ID allocation rule. |
| `temperature_min`, `temperature_max` | CALLER_PROVIDED | `CODE`: finite, positive, ordered, contiguous kelvin bounds. `PAIR`: ordinary upper bounds transfer; first lower bound still needs explicit source/target evidence. Added ID-5 bounds are unresolved for C3. |
| `enthalpy`, `entropy` | CALLER_PROVIDED | `CODE`: range-specific H/S constants at 298.15 K in group energy units. `PAIR`: ordinary conversion factors established; A entropy has one observed sign exception. |
| `coefficients[8]`, `powers[8]` | CALLER_PROVIDED | `CODE`: capacity of eight ordered Cp terms; `PAIR`: four fixed and three variable Legacy terms map to slots 0–6, with slot 7 zero in paired examples. The semantic plan takes ordered meaningful terms without native slot padding. Exact values/powers are retained; no refit, normalization, or tiny-term erasure. The native fill rule for unused slots remains `UNRESOLVED_BLOCKER` outside the paired seven-term profile. |
| `unknown_1[4]`, `padding_remaining[56]` | UNRESOLVED_BLOCKER | `UNKNOWN`: no fresh-output default. |

The rigorous plan always retains a base and A identity, even for a zero A.
`ExplicitZero` has no fabricated ID-5 interval: native zero-block encoding or
versioned omission policy is unresolved. A nonzero A may carry explicit ordered
H/S/Cp interval intent, but its Legacy leading-pair → ID-5 bounds, powers,
phase anchors, and exceptional entropy-sign behavior are not yet a general rule.

ID-10 comments and ID-11 extended-property records are
`NOT_REQUIRED_FOR_PROFILE`. Existing records remain `RAW_PRESERVED_ONLY` in the
parser/editor. Active ID-11 data is a typed blocker for fresh pure H/S/Cp plans.

## Exact evidence needed for FDB-C3

1. A controlled FactSage-created fresh FDB with a known single ordinary
   function and two Cp intervals, paired to its source intent, to establish
   header/shared-header/reserved/padding values, element IDs, charge, entry and
   reference numbers, timestamps, phase ID allocation, CP links, first lower
   bound, and ID-7 H/S selection.
2. A controlled paired Legacy → Modern export containing a nonzero A and an
   explicit zero A under the same FactSage version, to establish ID-5 bounds,
   default Cp powers, zero-object physical encoding/omission policy, and the
   known exceptional A entropy sign case. The exact source A lines and generated
   FDB/SLN references are needed; proprietary bytes remain local and uncommitted.
3. A source with deliberately active magnetic, pressure-volume, transition, or
   ID-11 data only if that family is to be admitted later. The present profile
   blocks such inputs rather than requiring those equations for C3.
