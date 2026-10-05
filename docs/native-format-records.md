# Native CDB/FDB records

Status: CURRENT NATIVE AUTHORITY.

This ledger centralizes established and unresolved facts about the shared Compound/Function Database binary family.

| ID | Subject | Scope | Authority | Native fact / policy |
| --- | --- | --- | --- | --- |
| CF-FAMILY-01 | CDB/FDB physical family | CDB/FDB | CONFIRMED | `.CDB` and `.FDB` use the same `CMPD` physical chunk family. File role is supplied by bundle context; filename/header alone is not sufficient semantic proof. |
| CF-CHUNK-01 | Physical framing | CDB/FDB | CONFIRMED | File is a flat sequence of 256-byte chunks. First chunk must be ID 9 for this parser family. Known IDs 1..11 retain their exact native IDs through round trip. |
| CF-CPKIND-01 | CP IDs 2–6 | heat-capacity ranges | PARTIAL | IDs 2–6 share the same physical layout and remain distinct native kinds. Their complete provider-wide semantic distinction is unresolved and must not be invented. Ordinary FDB validation requires homogeneous-kind sequences. |
| CF-FDB-CP-01 | Ordinary FDB H/S/Cp interpretation | Function-compatible ordinary phases | CONFIRMED / BOUNDED | Each CP range provides H and S constants referenced at 298.15 K plus `Cp(T)=sum(c_i T^p_i)`. Continuous adjacent ranges are required for the bounded FDB thermodynamic view. |
| CF-ENERGY-01 | Energy unit code | compound | CONFIRMED | Raw code 0 is calorie-based and converts by exactly 4.184; raw code 1 is joule-based; other values are preserved and conversion refuses them. |
| CF-PRESSURE-01 | Pressure unit code | compound | CONFIRMED / BOUNDED | Raw 0 = atmosphere, raw 1 = bar. This is a storage-unit code and does not establish thermodynamic reference pressure. |
| CF-PHASEID-01 | Phase state/index encoding | phase | CONFIRMED | raw >990 aqueous; >900 gas; >800 liquid; otherwise solid. State-local index subtracts 990/900/800/100 respectively. Suspicious nonpositive indexes are preserved with diagnostics. |
| CF-CHARGE-01 | Formula charge | shared record header | CONFIRMED | Shared native record header includes signed formula charge. Preserve exactly. |
| CF-DATE-01 | OLE Automation date | header/shared records | CONFIRMED | Native timestamps are stored as f64 OLE Automation dates. Raw value remains authoritative; typed conversion is optional. |
| CF-DENSITY-01 | Density / packed physical-family encoding | physical tail | PARTIAL / WARNING_ONLY | **ConfirmedByDomainExpert:** the physically realistic low portion carries ordinary density, while an integer offset greater than 100 is added to encode the advanced volumetric/physical equation-family selector. The exact selector values, family map, coefficient ownership, units, and equations are not yet established, and the current UI does not make the family-selection mechanism obvious. Preserve the full raw value and auxiliary coefficient array; never discard the high portion as numerical noise. |
| CF-ID11-01 | ID-11 conductivity range | phases | CONFIRMED / TRANSPORT-ONLY FOR THERMODYNAMICS | Linking by exact raw phase ID is established and lossless. Domain expert identifies ID-11 records as conductivity entries rather than thermodynamic state-function contributions. Preserve/filter them when operating on CDB/FDB bundles, but they are not required for thermodynamic conversion to ChemSage and must not block effective-G eligibility. Exact conductivity equation, units and coefficient meanings may remain opaque unless conductivity export is implemented. |
| CF-MAG-01 | Magnetic fixed fields | phase physical tail | PARTIAL | Native fields are preserved. Exact-zero fixed-slot pattern is inactive for bounded ordinary FDB eligibility; materially active values require external scientific semantics and currently block a claim of complete CP-backed G. |
| CF-PV-01 | Pressure/volume fixed fields | phase physical tail | PARTIAL | Native density/expansion/compressibility/bulk-derivative fields are preserved. Nonzero values prevent claiming the bounded CP-only effective G is complete until their equations are established. |
| CF-TRANS-01 | Transition phase | ID 8 | PARTIAL | Transition enthalpy, transition temperature and parent phase linkage are structurally typed. Effective-G chaining/entropy-jump semantics are not established. |
| CF-RAW-01 | Raw serialization authority | all | CONFIRMED | `RawDatabase` is the only serialization authority. Accepted unmodified input must serialize byte-for-byte identically, including unknown chunks, padding, reserved bytes and CP ID variants. |
| CF-FDBROLE-01 | Function-compatible profile evidence | FDB role | CONFIRMED / BOUNDED | `read_flag == 0` is an observed Function-compatible guardrail, not an intrinsic FDB classifier because valid CDBs can also have zero. Caller supplies logical bundle role. |

## Explicitly not established here

The following downstream research claims must not be treated as native authority until reproduced and recorded in this repository: a universal meaning for CP ID 2 versus 5, a fresh-writer rule selecting those IDs, semantics of opaque ID-9 fields beyond established layout, high-order density family selectors, or any guessed magnetic/volume equation. ID-11 is established as conductivity transport, but its conductivity equation/units remain unspecified unless that property is explicitly supported.

## Update rule

Native findings discovered while working in another repository must be copied here with their evidence and authority state before that downstream repository treats them as current parser knowledge.