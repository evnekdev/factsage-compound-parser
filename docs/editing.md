# Controlled editing

`edit::DatabaseEditor` owns exactly one authoritative `RawDatabase` and an optional lazy `DomainIndex`. It never holds a separately mutable semantic model. The byte-backed raw stream is the serialization source of truth; `DatabaseEditor::view()` rebuilds and borrows the semantic index when required.

```rust
use factsage_compound_parser::edit::DatabaseEditor;

fn rename(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = DatabaseEditor::from_path(input_path)?;
    editor.set_compound_name(0, "Example")?;
    let view = editor.view()?;
    assert_eq!(view.compound_count(), 1);
    drop(view);
    editor.write_to_path(output_path)?;
    Ok(())
}
```

The numerical arguments identify compounds and phases in indexed stream order. `DatabaseEditor::raw()` exposes the current immutable source of truth; `into_raw()` transfers it for direct serialization.

## Index invalidation

Supported setters change only established in-place fields and retain the current index:

- compound and phase names;
- ordinary phase 298 K enthalpy and entropy, supplied in SI units;
- transition enthalpy, supplied in SI units;
- transition temperature in kelvin;
- all seven real stoichiometric coefficients.

Low-level structural changes invalidate the cached index: `raw_mut`, `insert_chunk`, `push_chunk`, and `remove_chunk`. They preserve supplied raw chunks and their physical order but deliberately permit temporarily invalid semantic streams. The next `view()` or explicit `rebuild_index()` validates ordering. A failed rebuild preserves the raw stream and leaves no current cached index.

Borrowed views cannot coexist with a mutable editor borrow, so Rust prevents stale view references from surviving an edit.

## Setter behavior

Names must be ASCII, contain no embedded NUL, and fit their exact 40-byte field. The editor writes supplied bytes and NUL-pads only the unused portion of that selected field. It rejects overlong or non-ASCII text instead of truncating or decoding lossily.

Numeric setters reject non-finite values. SI energy setters use the owning compound's stored energy code: code 0 is divided by 4.184 before storage and code 1 is stored unchanged. Unknown energy codes return a typed error. The documented setter-locality tests verify that no unrelated bytes change.

## Deliberate limits

The editor does not change formulae, comments, density, unit codes, CP coefficients, CP stored enthalpy/entropy, kappa records, padding, reserved fields, or unknown chunks.

The Python implementation propagates edited ordinary-phase enthalpy and entropy into attached CP records. Rust deliberately does not: the CP anchor fields' reference convention remains unresolved, so copying phase values would be unverified semantic mutation. The Python transition-enthalpy setter divides by 4.184 unconditionally; Rust applies the actual compound energy code consistently.

Diagnostics and indexes never affect serialized bytes. Call `write_to`, `write_to_path`, or `into_raw().to_bytes()` to serialize the one authoritative raw stream.