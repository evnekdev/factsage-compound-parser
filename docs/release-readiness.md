# Release-readiness review

This pass establishes the repository foundation for future robustness work. It is not a production-readiness claim.

## Completed foundation

- dual license metadata: `MIT OR Apache-2.0`, with canonical `LICENSE-MIT` and `LICENSE-APACHE` files;
- Rust edition 2024 with a declared and tested MSRV of Rust 1.85.0;
- package metadata for description, repository, README, keywords, and categories;
- stable Linux quality CI, MSRV CI, Windows tests, and macOS tests;
- Dependabot updates for Cargo and GitHub Actions;
- scheduled `cargo audit` and pull-request dependency review;
- concise contribution and security policies;
- package-list and package-build validation;
- private fixture remains ignored, untracked, and unnecessary for CI.

## Packaging policy

The crate uses Cargo's default package inclusion rules. The package contains tracked Rust source, `Cargo.toml`, README, license files, public documentation, and the validated schema. `target`, ignored local CDB data, temporary parity output, and other untracked local files are not package inputs. The package is checked but not published.

The lockfile remains in the repository for reproducible development and CI resolution; consumers of this library do not need to use the repository lockfile when resolving a dependency.

## Remaining blockers

Before calling the crate production-ready or publishing it, complete:

- property testing and fuzzing for malformed, chunk-aligned, and adversarial input;
- broader malformed-input and corruption coverage across all public entry points;
- final public API-stability and versioning review;
- continued dependency, action, and security review;
- confirmation that the reverse-engineered semantics are sufficiently documented for the intended users.

Unknown fields, CP integration conventions, density units, kappa semantics, and unsupported thermodynamic meanings remain unresolved. The private fixture must stay local and must never be included in reports, issues, packages, or commits.