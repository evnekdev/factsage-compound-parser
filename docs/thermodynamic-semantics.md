# Thermodynamic semantics

The thermo module is a read-only layer over the raw and domain models. It
converts or evaluates established fields without changing preserved records.
It does not construct a cross-provider canonical Gibbs expression or evaluate
ID-11 physical-property equations.

## Evidence status

| Area | Status | Scope |
| --- | --- | --- |
| CDB/FDB physical family | established | Both use `CMPD` records. A zero `read_flag` is compatible with every locally examined FDB, but also occurs in valid CDBs. |
| FDB ordinary CP H/S/Cp definition | established | Finite, contiguous, one-kind CP sequences on ordinary phases whose independently integrated H/S values are continuous at shared boundaries. |
| Ordinary phase record H/S versus CP H/S | partially established | Both pairs are preserved and unit-convertible. They cannot be universally equated from the available evidence, so the provider thermodynamic view uses CP-range constants. |
| CP IDs 2–6 | partially established | Every examined FDB phase used one homogeneous kind; ID 2 had multi-range sequences and IDs 4/5 were observed as single-range sequences. The kind's wider provider meaning remains unknown. |
| Transition effective G | unresolved | Parent linkage is structural; an entropy-jump rule and chaining behavior are not established. |
| Pressure, magnetic, volume and ID-11 contributions | unresolved | Materially active values block complete effective-G eligibility; no equation is guessed. Exact-zero fixed physical-tail slots are inactive. |

The local evidence survey read installed files without embedding, printing, or
committing database records. It compared 298.15 K and 298 K integration
hypotheses at all available CP boundaries. The 298.15 K hypothesis reproduced
the stored cross-range H/S continuity; the 298 K alternative did not. No
ChemApp evaluator API was available to use as an additional oracle.

## Compound units

The established raw mappings are:

| Raw code | EnergyUnit | Meaning |
| ---: | --- | --- |
| 0 | `Calories` | Stored energy values use the calorie-based convention |
| 1 | `Joules` | Stored energy values are already joule-based |
| other | `Unknown(code)` | Preserved, but conversion returns a typed error |

The calorie conversion factor is exactly 4.184. `to_joules` multiplies calorie-based values and passes joule-based values through. `from_joules` performs the inverse operation. These views never modify raw compound, phase, or CP fields.

The pressure mappings established from documentation and local fixtures are raw code 0 for atmospheres and raw code 1 for bars. Unknown pressure values are preserved. No pressure conversion is implemented, and this storage-unit code must not be mistaken for evidence of the thermodynamic reference pressure.

## Database-profile evidence

`CompoundDatabaseProfileEvidence` is deliberately a validation result rather
than an FDB classifier:

- `FunctionCompatible` means the header has the observed FDB guardrail,
  `read_flag == 0`.
- `FunctionGuardrailMismatch` preserves a nonzero raw flag and reports that it
  does not match that guardrail.

The guardrail is necessary evidence when a caller has already assigned an FDB
logical bundle role. It is insufficient on its own because valid CDB files can
also have a zero flag. Header unknown bytes remain unknown and are not used.

## Phase thermodynamic accessors

Ordinary phases expose their preserved raw enthalpy and entropy through
`enthalpy_298_raw` and `entropy_298_raw`, plus unit-converting accessors.
Their exact relationship to every CP range is not assumed by the FDB view.
Transition phases expose stored transition enthalpy, transition temperature in
kelvin, and the preserved parent phase ID. Asking for an ordinary property on a
transition phase, or vice versa, returns `PhaseThermoError::WrongPhaseType`.

`CompoundView::fdb_phase_thermodynamic_view` is the provider-level API for an
explicitly assigned FDB role; callers first check
`DatabaseView::database_profile_evidence`. Its ordinary variant exposes each
range's H/S constants as values at 298.15 K; it does not need to infer them
from the ordinary phase record. Its transition variant exposes only the typed
parent relation.

The ordinary view's `effective_g_eligibility` must be checked before presenting
the CP-backed expression as a complete Gibbs function. `PureCpBacked` requires
all magnetic and pressure-volume fixed slots to be finite zero and no linked
ID-11 coefficient. A nonzero magnetic tuple, nonzero density/expansion/
compressibility/bulk-derivative data, or active ID-11 coefficient returns a
typed blocker. ID-11 bounds and powers are structural rather than contributions
by themselves. A non-finite physical value is `PendingProviderEvidence`. This
classification does not derive any physical equation.

## OLE Automation dates

`OleAutomationDate` retains the original finite raw day count. Conversion uses the OLE epoch of 1899-12-30 and the Windows/OLE negative-fraction rule. Conversion rounds to the nearest millisecond with the same signed rounding and negative-fraction behavior as Windows `System.DateTime.FromOADate`. Non-finite values and values outside the representable OLE range return `DateError`.

## Density

The density convenience accessor follows the established Python expression:

```text
density = density_raw % 1_000_000
```

Rust floating-point remainder is used deliberately. The returned value has no asserted physical unit and the upper encoded portion remains reverse-engineered. Non-finite raw values return `DensityError`.

## Heat capacity and established ordinary FDB semantics

Each range evaluates the preserved eight coefficient/power pairs explicitly:

```text
Cp(T) = sum(coefficients[i] * T.powf(powers[i]))
```

For the validated FDB ordinary subset, the CP record also defines a complete
range-local H/S representation:

```text
H_r(T) = H_r(298.15 K) + integral(298.15 K..T, Cp_r(t) dt)
S_r(T) = S_r(298.15 K) + integral(298.15 K..T, Cp_r(t) / t dt)
G_r(T) = H_r(T) - T S_r(T)
```

For a term `c T^p`, H uses `c/(p+1) * (T^(p+1)-T0^(p+1))`, with
`c ln(T/T0)` at `p = -1`. S uses
`c/p * (T^p-T0^p)`, with `c ln(T/T0)` at `p = 0`.

The stored H/S constants are **per-range 298.15 K constants**, not anchors at
`Tmin` or `Tmax`. Adjacent source-order ranges at a shared boundary carry
constants adjusted so their derived H and S are continuous. CP may be
discontinuous at a boundary. This was verified against all available finite
contiguous FDB range pairs with a relative numerical residual check; it is not
inferred from the field names.

Construction enforces that claim. At every exact shared source boundary it
evaluates each range independently from its own 298.15 K anchors, compares H,
then compares S, and returns `EnthalpyDiscontinuity` or
`EntropyDiscontinuity` with both source chunks and numerical diagnostics. The
provider-only allowance in the compound's native energy unit is:

```text
1e-8 + 1e-8 * max(abs(left), abs(right))
```

This tolerance is not reused for composition algebra, GUI display, or
cross-provider scientific comparison.

`PhaseHeatCapacityRangeView` exposes analytical Cp/H/S/G evaluation within its
stored interval. For every finite power `p`, including negative, fractional,
and positive cases, H integrates `c*T^p`; `p = -1` uses `c*ln(T/T0)`. S
integrates `c*T^(p-1)`; `p = 0` uses `c*ln(T/T0)`. Positive absolute
temperature is required, non-finite mathematical results are typed errors, and
public evaluation never extrapolates.

The validated view treats individual bounds as closed. At a shared boundary,
the lower range is selected by the legacy direct evaluator; the established H/S
continuity makes either range's derived G equal there. It rejects zero-width,
nonfinite, overlapping, reordered, mixed-kind, and gapped ranges for
ordinary-phase effective-G use. It never extrapolates outside stored support.
The lower-bound/upper-bound behavior of arbitrary malformed or future files is
therefore a typed validation result, not a repaired function.

The general stored-CP evaluator retains its existing inclusive-bound behavior:
`t_min <= T <= t_max`; it reports true overlaps and no matching range rather
than silently selecting an unrelated record.

## Transitions

ID-8 gives a transition phase, a parent raw phase ID, a transition temperature,
and a transition enthalpy in the compound energy unit. Exact raw-ID parent
linking is established and exposed through `TransitionParentRelation`.

No locally available FDB transition record, external evaluator result, or
format documentation established whether `DeltaS = DeltaH / T_transition`,
how multiple transitions chain, whether a transition inherits or replaces CP
ranges, or whether the jump is already represented in CP constants. The parser
therefore does not calculate transition H/S/G and downstream consumers must
keep transition effective-G support pending.

## ID-11 records are not thermodynamic evaluations

`PhysicalPropertyRangeView` exposes structurally parsed ID-11 records and their exact raw phase-ID links. Windows corpus validation confirms 86 such records preserve and link correctly, with finite observed numeric fields and valid bounds. This does not establish a kappa equation, a pressure or temperature convention, a density relation, units, or meanings for the coefficient arrays. No ID-11 evaluation API is provided.

## Compatibility and scope

The Rust layer follows the Python getter conversion rule consistently for
ordinary and transition enthalpy. The raw-authoritative editor intentionally
does not copy the Python transition setter asymmetry: it converts according to
the actual energy code. It does not propagate ordinary phase edits into CP
constants because the records remain independent raw values even though the CP
constants' 298.15 K role is established. The parser exposes provider semantics;
cross-provider unit normalization and canonical Gibbs materialization belong to
the consuming comparison layer.
