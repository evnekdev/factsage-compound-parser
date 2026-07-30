# Python parity validation

The normal Rust test suite never requires Python or a proprietary fixture. A separate aggregate-only comparison path checks the current Rust library against the original `factsage-compound` parser when both are available locally.

Set the Python checkout path, then run the gated test:

```powershell
$env:FACTSAGE_COMPOUND_PYTHON_ROOT = 'C:\path\to\factsage-compound'
$env:FACTSAGE_COMPOUND_PYTHON_EXECUTABLE = 'python' # optional
cargo test --test python_parity
```

The test invokes `scripts/python_parity.py`, which dynamically loads only the original parser module. It requires that checkout's NumPy and pyparsing dependencies. It reads the CDB but never calls the Python `save` method, modifies the fixture, or prints record content.

It compares only aggregate values: file size, chunk count and ID histogram, grouped record counts, orphan and duplicate counts, phase-state histogram, unit histograms, invalid timestamp/density/bound counts, and CP interval topology. The private fixture remains ignored and is skipped if absent.

## Latest local comparison

For the ignored local `MS16BASE.CDB`, Rust and Python agreed on all compared aggregates:

| Metric | Result |
| --- | ---: |
| File size / chunks | 755,712 bytes / 2,952 |
| Compounds | 537 |
| Ordinary / transition phases | 620 / 64 |
| CP / kappa / comment chunks | 1,518 / 0 / 212 |
| Orphan ranges / duplicate phase IDs | 0 / 0 |
| Phase states (solid / liquid / gas / aqueous) | 583 / 51 / 48 / 2 |
| Energy units (code 0 / code 1) | 183 / 354 |
| Pressure units (code 0 / code 1) | 417 / 120 |
| Invalid timestamps, densities, CP bounds | 0 / 0 / 0 |
| Strict CP overlaps / gaps / shared endpoints | 0 / 0 / 834 |

The shared endpoints are intentional adjacent bounds, not strict overlaps. The Rust phase-level selector uses the lower-temperature range at such a point; see [thermodynamic semantics](thermodynamic-semantics.md).

## Known behavioural differences

The Python parser assumes valid ordering, does not retain unknown chunks in its grouped model, and does not diagnose duplicate or orphan links. Rust preserves those records and reports typed diagnostics, so pathological input is intentionally not expected to have identical semantic results.

Rust also corrects the Python transition-enthalpy setter's unconditional 4.184 division and does not propagate ordinary phase setter values into CP anchor fields; both deviations avoid unverified semantic mutation.
