# Semantic decoding rules

This document records behaviour implemented by the completed Python project and separates it from unresolved reverse-engineering assumptions.

## Fixed strings

Known textual fields are fixed-width ASCII:

- database comment: 80 bytes;
- compound name: 40 bytes;
- compound formula: 40 bytes;
- phase name: 40 bytes;
- comment fragment: 80 bytes;
- reserved compound strings: 40 and 12 bytes.

The Python code uses `decode("ascii").strip()`, which removes both leading and trailing ASCII whitespace but does not explicitly remove NUL bytes. For a safer Rust API:

1. preserve the original byte array;
2. for display, truncate at the first NUL if present;
3. remove trailing NUL and space padding;
4. avoid removing meaningful leading spaces unless tests confirm they are never used;
5. expose a lossy display method separately from strict ASCII decoding.

## OLE Automation dates

The database date and shared-header timestamp occupy eight bytes. The wiki identifies them as Windows OLE Automation dates, represented as an IEEE-754 `f64` count of days relative to the OLE epoch.

The binary layer should expose the raw `f64`. Date conversion should be optional because:

- negative values and the historical OLE date discontinuity need deliberate handling;
- corrupt or sentinel values should remain inspectable;
- no date library should be forced into the core parser unnecessarily.

## Energy units

The compound chunk contains a four-byte `unit_energy` code. The Python semantic layer implements one confirmed rule:

```text
unit_energy == 0  => stored energy values are calories-based
                     multiply by 4.184 when exposing J/mol or J/mol/K
otherwise         => expose stored values unchanged
```

This conversion is applied to ordinary-phase enthalpy and entropy. Setters divide by `4.184` before storing when `unit_energy == 0`.

The Python transition-enthalpy setter always divides by `4.184`, although its getter only multiplies when `unit_energy == 0`. This asymmetry is likely a bug and must not be copied blindly. The Rust implementation should apply one consistent conversion policy and verify it against sample files.

CP-range enthalpy and entropy properties in Python return raw stored values
without applying the compound energy-unit conversion. The Rust provider view
uses the same compound-level code for their explicit conversions; installed FDB
records use the established joule code. This does not imply that all future or
arbitrary CMPD files do.

## Pressure units

The compound chunk contains a four-byte `unit_pressure` code. The older wiki labels the choices as atmosphere and bar, but the Python implementation does not interpret the values. Preserve the raw integer until sample-based mapping is established.

## Phase IDs, state and index

The Python parser derives phase state from `phase_id_raw`:

```text
phase_id_raw > 990  => aqueous
phase_id_raw > 900  => gas
phase_id_raw > 800  => liquid
otherwise           => solid
```

It derives the state-local index as:

```text
aqueous: phase_id_raw - 990
gas:     phase_id_raw - 900
liquid:  phase_id_raw - 800
solid:   phase_id_raw - 100
```

Examples inferred from those rules:

| Raw ID | State | Index | ChemApp label |
|---:|---|---:|---|
| 101 | solid | 1 | `s` |
| 102 | solid | 2 | `s2` |
| 801 | liquid | 1 | `l` |
| 901 | gas | 1 | `g` |
| 991 | aqueous | 1 | `aq` |

The comparisons are strict `>` operations in Python. Boundary values such as `800`, `900`, and `990` would fall into the preceding category and may be invalid or reserved. Rust should validate these boundaries rather than silently assigning them.

## ChemApp-style names

The helper parser expects identifiers conceptually shaped like:

```text
FORMULA_NAME(state-and-index)
```

Examples include `FeO_wustite(s)` and labels such as `s`, `s2`, `l`, `g`, or `aq`. The active helper separates:

- formula: text before the first underscore;
- name: text between that underscore and the final opening parenthesis;
- label: text inside the final parentheses.

This is convenience logic, not part of the `.CDB` binary layout.

## Density encoding

The Python property returns:

```text
density = density_raw % 1_000_000
```

Its setter preserves the million-multiple portion and replaces the remainder:

```text
density_raw = trunc(density_raw / 1_000_000) * 1_000_000 + density
```

This implies that `density_raw` packs density together with another code in the higher decimal digits. The meaning and units of that high component are unresolved. A Rust parser should expose:

- `density_raw: f64`;
- an explicitly provisional convenience accessor for the remainder;
- no claim about physical units without further verification.

Using floating-point modulo for an encoded decimal field is unusual, so inspect real values before stabilising this API.

## Heat-capacity expression

Each CP record stores eight coefficient/power pairs:

```text
Cp(T) = Σ a_i T^(p_i), i = 0..7
```

The range applies between `t_min` and `t_max` in kelvin. For the validated
ordinary FDB subset, stored H/S are independent values at 298.15 K for the
range's own Cp expression. They are neither `Tmin` nor `Tmax` anchors. Between
adjacent contiguous source-order ranges, their values satisfy H/S continuity
when each range is integrated from 298.15 K.

The ordinary phase's separate H/S fields must not be propagated into every CP
record: local FDB evidence does not establish a universal equality. The
provider thermodynamic view therefore uses CP H/S plus Cp directly and rejects
missing, gapped, overlapping, reordered, mixed-kind, zero-width, or nonfinite
ranges, and adjacent ranges with discontinuous integrated H or S, instead of
guessing a repair or extrapolation. The continuity check uses an explicit
provider-format tolerance independent of downstream comparison tolerances.

The ordinary phase view also classifies whether H/S/Cp is a complete
effective-G representation. All-zero fixed physical-tail fields are inactive;
materially nonzero magnetic, pressure-volume, or ID-11 data blocks completeness.
ID-11 bounds and powers alone are structural; a nonzero coefficient is active.
Non-finite evidence remains pending. A pressure unit code identifies only
storage units and does not establish a reference pressure.

Validation suggestions:

- require finite coefficients and powers for numerical evaluation;
- permit unused zero terms;
- do not assume conventional integer or half-integer powers;
- individual validated FDB range bounds are closed; at a shared boundary the
  existing direct evaluator selects the lower range, while derived H/S/G are
  continuous;
- never extrapolate past stored range support.

## Transition phases

ID 8 replaces ordinary `enthalpy` and `entropy` with:

- transition enthalpy;
- transition temperature;
- parent phase raw ID;
- current phase raw ID.

The remaining physical-property fields have the same layout as an ordinary
phase. A transition phase is linked to its parent by exact raw phase ID through
`TransitionParentRelation`. The physical transformation rule remains unknown:
do not infer a transition entropy, apply `DeltaH` twice, chain records, or
inherit Cp ranges until independent provider evidence establishes those rules.

## Extended properties (ID 11)

The field names inherited from Python suggest a combination of temperature and pressure functions:

- `f1_t_coefficients[10]`, `f1_t_powers[8]`;
- `f2_p_coefficients[3]`, `f2_p_powers[2]`;
- `f3_t_coefficients[5]`, `f3_t_powers[3]`.

The unequal coefficient/power counts indicate that some coefficients may be constants, integration terms, or parameters outside a simple power series. Do not construct a formula from the names alone. Preserve and document the arrays until behaviour is independently established.

## Unresolved fields

The following should remain raw and clearly marked as unknown or reserved:

- header padding and unknown byte regions;
- the two `reference` values and `entry_number` semantics;
- compound reserved strings;
- compound four-byte unknown field;
- pressure-unit codes;
- phase `phase_id_raw_neg` meaning;
- encoded high portion of `density_raw`;
- exact semantics distinguishing CP IDs 2–6;
- CP four-byte unknown field;
- ID-11 formula and property meaning.

A knowledge base should distinguish facts derived from executable code from hypotheses derived from UI labels or field names.
