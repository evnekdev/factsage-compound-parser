# Byte-accurate chunk layouts

All offsets below are relative to the beginning of the **256-byte chunk**, including the one-byte chunk ID. Multi-byte values are little-endian.

## Primitive notation

| Notation | Meaning |
|---|---|
| `u1`, `s1` | unsigned/signed 8-bit integer |
| `u2`, `s4` | unsigned 16-bit / signed 32-bit integer |
| `u4` | unsigned 32-bit integer |
| `f4`, `f8` | IEEE-754 binary32 / binary64 |
| `bytes[n]` | uninterpreted fixed-length bytes |
| `ascii[n]` | fixed-length ASCII field, normally NUL/space padded |

## Database header — ID 9

| Offset | Size | Type | Field |
|---:|---:|---|---|
| 0 | 1 | u1 | `chunk_id = 9` |
| 1 | 1 | u1 | `padding1` |
| 2 | 4 | ascii[4] | `magic`, expected `CMPD` |
| 6 | 2 | bytes[2] | `padding2` |
| 8 | 8 | f8 | `date`, OLE Automation date |
| 16 | 1 | u1 | `read_flag` |
| 17 | 11 | bytes[11] | `unknown1` |
| 28 | 80 | ascii[80] | `comment` |
| 108 | 136 | bytes[136] | `padding3` |
| 244 | 12 | bytes[12] | `unknown2` |

`read_flag == 0` is an observed FDB-compatible guardrail, not an intrinsic
CDB/FDB classifier: valid CDB files can also use it. `unknown1` and `unknown2`
remain uninterpreted and are not profile evidence.

## Shared entry header — chunks 1–8, 10, 11

The shared header begins at chunk offset 1 and occupies 31 bytes.

| Offset | Size | Type | Field |
|---:|---:|---|---|
| 1 | 7 | u1[7] | `element_ids` |
| 8 | 1 | u1 | `padding_coeff` |
| 9 | 7 | u1[7] | `element_coeffs` |
| 16 | 1 | s1 | `charge_raw` |
| 17 | 1 | u1 | `entry_number` |
| 18 | 4 | u2[2] | `reference` |
| 22 | 8 | f8 | `timestamp`, OLE Automation date |
| 30 | 2 | bytes[2] | `unknown1` |

All chunk-specific fields below therefore start at offset 32.

## Compound — ID 1

| Offset | Size | Type | Field |
|---:|---:|---|---|
| 0 | 1 | u1 | `chunk_id = 1` |
| 1 | 31 | shared | `header` |
| 32 | 40 | ascii[40] | `compound_name` |
| 72 | 40 | ascii[40] | `reserved_string1` |
| 112 | 40 | ascii[40] | `formula_name` |
| 152 | 4 | bytes[4] | `unknown` |
| 156 | 4 | u4 | `unit_energy` |
| 160 | 4 | u4 | `unit_pressure` |
| 164 | 12 | ascii[12] | `reserved_string2` |
| 176 | 56 | f8[7] | `coeff_real` |
| 232 | 24 | bytes[24] | `padding_final` |

The older wiki/Kaitai draft incorrectly merged the 4-byte unknown field and both 4-byte unit fields into a 12-byte unknown area. The completed Python dtype establishes them as three separate fields.

## Ordinary phase — ID 7

| Offset | Size | Type | Field |
|---:|---:|---|---|
| 0 | 1 | u1 | `chunk_id = 7` |
| 1 | 31 | shared | `header` |
| 32 | 8 | f8 | `enthalpy` |
| 40 | 8 | f8 | `entropy` |
| 48 | 4 | s4 | `phase_id_raw_neg` |
| 52 | 4 | s4 | `phase_id_raw` |
| 56 | 8 | f8 | `density_raw` |
| 64 | 16 | f4[4] | `thermal_expansion_coeffs` |
| 80 | 16 | f4[4] | `compressibility_coeffs` |
| 96 | 8 | f4[2] | `bulk_modulus_derivative_coeffs` |
| 104 | 4 | f4 | `magnetic_temperature` |
| 108 | 4 | f4 | `magnetic_moment` |
| 112 | 4 | f4 | `p_factor` |
| 116 | 20 | bytes[20] | `padding1` |
| 136 | 40 | ascii[40] | `phase_name` |
| 176 | 80 | bytes[80] | `padding2` |

## Transition phase — ID 8

The layout is identical to ID 7 except for offsets 32–55:

| Offset | Size | Type | Field |
|---:|---:|---|---|
| 32 | 8 | f8 | `transition_enthalpy` |
| 40 | 8 | f8 | `transition_temperature` |
| 48 | 4 | s4 | `parent_phase_id_raw` |
| 52 | 4 | s4 | `phase_id_raw` |

Offsets 56–255 use the same density, physical coefficients, name and padding layout as the ordinary phase.

## Heat-capacity range — IDs 2, 3, 4, 5, 6

| Offset | Size | Type | Field |
|---:|---:|---|---|
| 0 | 1 | u1 | one of IDs 2–6 |
| 1 | 31 | shared | `header` |
| 32 | 8 | f8 | `enthalpy` |
| 40 | 8 | f8 | `entropy` |
| 48 | 4 | s4 | `phase_id_raw` |
| 52 | 4 | bytes[4] | `unknown1` |
| 56 | 8 | f8 | `t_min` |
| 64 | 8 | f8 | `t_max` |
| 72 | 64 | f8[8] | `cp_coefficients` |
| 136 | 64 | f8[8] | `powers` |
| 200 | 56 | bytes[56] | `padding_remaining` |

The represented expression is:

```text
Cp(T) = sum(i = 0..7, cp_coefficients[i] * T ^ powers[i])
```

For a validated ordinary FDB CP sequence, fields `enthalpy` and `entropy` are
the range's H/S constants at 298.15 K. They are range-specific constants, not
`Tmin` or `Tmax` values. This interpretation is not retroactively imposed on
unvalidated CDB or malformed streams.

## Comment fragment — ID 10

| Offset | Size | Type | Field |
|---:|---:|---|---|
| 0 | 1 | u1 | `chunk_id = 10` |
| 1 | 31 | shared | `header` |
| 32 | 80 | ascii[80] | `comment` |
| 112 | 144 | bytes[144] | `padding_remaining` |

Long comments are represented by consecutive ID-10 chunks and should be concatenated in stream order after decoding and trimming each fragment.

## Extended physical-property / kappa range — ID 11

| Offset | Size | Type | Field |
|---:|---:|---|---|
| 0 | 1 | u1 | `chunk_id = 11` |
| 1 | 31 | shared | `header` |
| 32 | 8 | f8 | `t_min` |
| 40 | 8 | f8 | `t_max` |
| 48 | 4 | s4 | `phase_id_raw` |
| 52 | 4 | bytes[4] | `unknown1` |
| 56 | 80 | f8[10] | `f1_t_coefficients` |
| 136 | 32 | f4[8] | `f1_t_powers` |
| 168 | 24 | f8[3] | `f2_p_coefficients` |
| 192 | 8 | f4[2] | `f2_p_powers` |
| 200 | 40 | f8[5] | `f3_t_coefficients` |
| 240 | 12 | f4[3] | `f3_t_powers` |
| 252 | 4 | bytes[4] | `padding_remaining` |

The Python implementation preserves these records but does not yet expose a semantic evaluator. The Rust parser should initially retain all arrays losslessly and defer interpretation until verified against FactSage behaviour or additional samples.
