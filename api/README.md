# Public API baselines

`0.1.0.txt` is the reviewed public API listing for the experimental 0.1.0
release. It contains only the output of `cargo-public-api` and no fixture,
corpus, machine path, or generated record data.

The snapshot was generated with cargo-public-api 0.52.0:

```powershell
cargo public-api --simplified --color never
```

After the `v0.1.0` tag exists, compare a later checkout against the release
tag with the tool's supported commit syntax:

```powershell
cargo public-api diff v0.1.0..HEAD
```

The crate follows an experimental 0.x compatibility policy: `0.1.x` may
evolve, but breaking changes must be explicitly documented and avoidable
breaking changes should not be silently introduced in patch releases.
