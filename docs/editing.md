# Controlled editing

`edit::DatabaseEditor` is an owned editor over one authoritative `RawDatabase`. It changes byte-backed raw fields and rebuilds `domain::Database` views on demand. It does not keep a second mutable domain copy.

```rust
use factsage_compound_parser::edit::DatabaseEditor;

let mut editor = DatabaseEditor::from_path(input_path)?;
editor.set_compound_name(0, "Example")?;
editor.set_phase_name(0, 0, "solid")?;
editor.set_ordinary_phase_enthalpy_298_j_per_mol(0, 0, -12_500.0)?;
let grouped = editor.domain()?;
editor.write_to_path(output_path)?;
```

The numerical indexes identify compounds and phases in their flat stream order. `DatabaseEditor::raw` exposes the current raw source of truth, and `into_raw` transfers it for direct serialization.

## Supported setters

- compound name;
- ordinary and transition phase name;
- ordinary phase 298 K enthalpy and entropy, supplied in SI units;
- transition enthalpy, supplied in SI units;
- transition temperature in kelvin;
- all seven real stoichiometric coefficients.

Names must be ASCII, contain no embedded NUL, and fit their exact 40-byte field. The editor writes the supplied bytes then NUL-pads the remainder of that field. It rejects overlong and non-ASCII names instead of truncating or lossy encoding them. No bytes outside the selected field are changed.

Numeric setters reject non-finite values. SI energy setters use the owning compound's raw energy code: code 0 is divided by 4.184 before storage, and code 1 is stored unchanged. Unknown codes return a typed error.

## Deliberate limits

The editor does not currently change formulae, comments, density, unit codes, CP coefficients, CP stored enthalpy/entropy, kappa records, padding, reserved fields, or unknown chunks.

The Python ordinary-phase setters propagate edited enthalpy and entropy to attached CP records. This crate deliberately does not: the CP anchor fields' reference convention is still unresolved, so copying phase values into them would be a semantic change without sufficient evidence. The Python transition-enthalpy setter always divides by 4.184, even for joule-coded compounds; the Rust editor instead applies the documented unit code consistently.

After a raw edit, call `DatabaseEditor::domain` or `into_domain` to rebuild links, lookups, and diagnostics. Diagnostics do not affect serialized bytes.
