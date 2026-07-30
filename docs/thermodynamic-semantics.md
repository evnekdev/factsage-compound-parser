# Thermodynamic semantics

The thermo module is a read-only layer over the raw and domain models. It converts or evaluates established fields without changing preserved records. It does not integrate heat capacity, calculate enthalpy or entropy at arbitrary temperatures, calculate Gibbs energy, or evaluate ID-11 physical-property equations.

## Compound units

The established raw mappings are:

| Raw code | EnergyUnit | Meaning |
| ---: | --- | --- |
| 0 | `Calories` | Stored energy values use the calorie-based convention |
| 1 | `Joules` | Stored energy values are already joule-based |
| other | `Unknown(code)` | Preserved, but conversion returns a typed error |

The calorie conversion factor is exactly 4.184. `to_joules` multiplies calorie-based values and passes joule-based values through. `from_joules` performs the inverse operation. These views never modify raw compound, phase, or CP fields.

The pressure mappings established from documentation and local fixtures are raw code 0 for atmospheres and raw code 1 for bars. Unknown pressure values are preserved. No pressure conversion is implemented.

## Phase thermodynamic accessors

Ordinary phases expose stored raw enthalpy and entropy through `enthalpy_298_raw` and `entropy_298_raw`, plus explicit SI accessors. Transition phases expose stored transition enthalpy, transition temperature in kelvin, and the preserved parent phase ID. Asking for an ordinary property on a transition phase, or vice versa, returns `PhaseThermoError::WrongPhaseType`.

CP stored enthalpy and entropy fields are exposed conservatively as `stored_enthalpy_raw` and `stored_entropy_raw`. Their reference-temperature interpretation remains unverified.

## OLE Automation dates

`OleAutomationDate` retains the original finite raw day count. Conversion uses the OLE epoch of 1899-12-30 and the Windows/OLE negative-fraction rule. Conversion rounds to the nearest millisecond with the same signed rounding and negative-fraction behavior as Windows `System.DateTime.FromOADate`. Non-finite values and values outside the representable OLE range return `DateError`.

## Density

The density convenience accessor follows the established Python expression:

```text
density = density_raw % 1_000_000
```

Rust floating-point remainder is used deliberately. The returned value has no asserted physical unit and the upper encoded portion remains reverse-engineered. Non-finite raw values return `DensityError`.

## Heat capacity

Each range evaluates the preserved eight coefficient/power pairs explicitly:

```text
Cp(T) = sum(coefficients[i] * T.powf(powers[i]))
```

Evaluation requires finite positive kelvin temperature. Individual bounds are inclusive: `t_min <= T <= t_max`. Phase selection searches all ranges in stream order. If no range contains the temperature, the error reports available intervals. At an exact endpoint shared by two adjacent ranges, phase-level evaluation selects the lower-temperature range. Other multiple-range cases report all candidates.

## ID-11 records are not thermodynamic evaluations

`PhysicalPropertyRangeView` exposes structurally parsed ID-11 records and their exact raw phase-ID links. Windows corpus validation confirms 86 such records preserve and link correctly, with finite observed numeric fields and valid bounds. This does not establish a kappa equation, a pressure or temperature convention, a density relation, units, or meanings for the coefficient arrays. No ID-11 evaluation API is provided.

## Compatibility and scope

The Rust layer follows the Python getter conversion rule consistently for ordinary and transition enthalpy. The raw-authoritative editor intentionally does not copy the Python transition setter asymmetry: it converts according to the actual energy code. It also does not propagate ordinary phase edits into CP stored anchors because their reference convention remains unverified. CP integration and inferred density units remain unsupported.