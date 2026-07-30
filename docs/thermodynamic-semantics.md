# Thermodynamic semantics

The thermo module is a read-only layer over the raw and domain models. It converts or evaluates established fields without changing the preserved records. It does not integrate heat capacity, calculate enthalpy or entropy at arbitrary temperatures, calculate Gibbs energy, evaluate kappa, or serialize files.

## Compound units

The established raw mappings are:

| Raw code | EnergyUnit | Meaning |
| ---: | --- | --- |
| 0 | `Calories` | Stored energy values use the calorie-based convention |
| 1 | `Joules` | Stored energy values are already joule-based |
| other | `Unknown(code)` | Preserved, but conversion returns a typed error |

The calorie conversion factor is exactly 4.184. `to_joules` multiplies calorie-based values and passes joule-based values through. `from_joules` performs the inverse operation. These views never modify raw compound, phase, or CP fields.

The pressure mappings established from the documentation and local fixture are raw code 0 for atmospheres and raw code 1 for bars. Unknown pressure values are preserved. No pressure conversion is implemented in this milestone.

## Phase thermodynamic accessors

Ordinary phases expose stored raw enthalpy and entropy through `enthalpy_298_raw` and `entropy_298_raw`, plus explicit SI accessors. Transition phases expose stored transition enthalpy, transition temperature in kelvin, and the preserved parent phase ID. Asking for an ordinary property on a transition phase, or vice versa, returns `PhaseThermoError::WrongPhaseType`.

The CP-range stored enthalpy and entropy fields are exposed conservatively as `stored_enthalpy_raw` and `stored_entropy_raw`. Their exact reference-temperature interpretation remains unverified. Optional SI views apply the compound energy-unit conversion but do not claim a reference convention.

## OLE Automation dates

`OleAutomationDate` retains the original finite raw day count. Conversion uses the OLE epoch of 1899-12-30 and the Windows/OLE negative-fraction rule. The conversion rounds to the nearest millisecond with the same signed rounding and negative-fraction rule as Windows `System.DateTime.FromOADate`. Non-finite values and values outside the representable OLE range return `DateError`.

The raw database header and every shared entry header expose date helpers. Domain compound, phase, CP range, and physical-property range wrappers expose the corresponding shared timestamp helper; raw comment and kappa records expose theirs directly.

## Density

The phase density convenience accessor follows the established Python expression:

~~~text
density = density_raw % 1_000_000
~~~

Rust floating-point remainder is used deliberately. Inspection of the local fixture found 684 finite, positive phase values, none with a million-scale encoded portion or negative remainder, so no alternate integer or Euclidean rule was needed for this milestone. The returned value has no asserted physical unit, and the upper encoded portion remains reverse-engineered. Non-finite raw values return `DensityError`.

## Heat capacity

Each range evaluates the preserved eight coefficient/power pairs explicitly:

~~~text
Cp(T) = sum(coefficients[i] * T.powf(powers[i]))
~~~

Evaluation requires finite positive kelvin temperature. This is a deliberate physical-domain rule; it avoids ambiguous zero or negative bases for fractional and negative powers. Individual bounds are inclusive: `t_min <= T <= t_max`.

Phase selection searches all ranges in preserved stream order. Unsorted ranges are supported. If no range contains the temperature, the error reports the available intervals. At an exact endpoint shared by exactly two adjacent ranges, phase-level evaluation selects the lower-temperature range. This is supported by 834 such endpoints and zero strict overlaps in the local fixture. Other multiple-range cases report all candidate indexes; the first range is never silently selected. Invalid range bounds, non-finite coefficients or powers, invalid power terms, non-finite terms, and non-finite sums are typed errors.

`Compound::heat_capacity_at` selects a phase by index and derives the energy unit from that compound. The direct range methods accept an explicit `EnergyUnit` so callers can evaluate a detached range without introducing ownership or reference cycles. SI CP conversion uses the same 4.184 factor for calorie-based values.

## Compatibility and scope

The Rust layer follows the Python getter conversion rule consistently for ordinary and transition enthalpy. The raw-authoritative editor intentionally does not copy the Python transition setter asymmetry: it converts according to the actual energy code. It also does not propagate ordinary phase edits into CP stored anchors, because their reference convention remains unverified. CP integration and inferred density units remain unsupported.
