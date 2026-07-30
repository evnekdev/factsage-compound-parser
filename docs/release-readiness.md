# Release-readiness review

This pass establishes a strong library foundation. It is not a production-readiness or publication claim.

## Completed foundation

- dual license metadata: `MIT OR Apache-2.0`, with canonical `LICENSE-MIT` and `LICENSE-APACHE` files;
- Rust edition 2024 with declared and tested MSRV Rust 1.85.0;
- package metadata for description, repository, README, keywords, and categories;
- stable Linux quality CI, MSRV CI, Windows tests, and macOS tests;
- Dependabot updates for Cargo and GitHub Actions;
- scheduled `cargo audit` and pull-request dependency review;
- concise contribution and security policies;
- package-list and package-build validation;
- an owned raw-authoritative architecture with a non-duplicating semantic index and borrowed views;
- synthetic Criterion benchmarks, bounded property tests, and synthetic-seed-free cargo-fuzz targets;
- comprehensive public Rustdoc checked with warnings promoted to errors;
- local private-fixture validation that remains optional, ignored, in-memory, and aggregate-only.

## Packaging policy

The crate uses Cargo's default package inclusion rules. The package contains tracked Rust source, `Cargo.toml`, README, license files, public documentation, and the validated schema. Root build output, fuzz build output, generated fuzz lockfiles, ignored local CDB data, temporary parity output, and other untracked local files are not package inputs. The package is checked but not published.

The lockfile remains in the repository for reproducible development and CI resolution; consumers of this library do not need the repository lockfile when resolving a dependency.

## Remaining blockers

Before calling the crate production-ready or publishing it, complete:

- sustained sanitizer-backed fuzzing on a Linux/WSL runner, plus triage of any findings;
- broader malformed-input and corruption coverage across all public entry points;
- additional property cases for document-level and thermodynamic edge conditions;
- a final public API-stability, semver, and error-model review;
- continuing dependency, action, and security review;
- confirmation that reverse-engineered semantics are sufficiently documented for intended users.

Unknown fields, CP integration conventions, density units, kappa semantics, and unsupported thermodynamic meanings remain unresolved. The private fixture must stay local and must never be included in reports, issues, packages, fuzz corpora, or commits.