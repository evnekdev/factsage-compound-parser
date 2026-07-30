# Release-readiness review

This repository is a well-tested library foundation, not yet a production-ready public release.

## Present strengths

- native safe Rust parser with typed, structured errors;
- lossless raw serialization verified with synthetic NaN/infinity cases and the private fixture;
- explicit raw, domain, thermo, and editor module boundaries;
- aggregate-only private-fixture and Python-parity checks;
- minimal dependency set (`encoding_rs` for Windows-1252 comments);
- no `unsafe` code;
- no embedded proprietary data.

## Required before publication

- choose and add an explicit license;
- set and test an MSRV policy;
- add public CI for formatting, Clippy, tests, and rustdoc;
- add CI version coverage and an automated security/dependency review process;
- review package versioning and API stability guarantees;
- add adversarial/fuzz tests for malformed but chunk-aligned input;
- decide whether public raw-edit APIs should stabilize at the current field set;
- document governance, support, and security-contact policy if distributed broadly.

`Cargo.toml` includes a package description, readme, repository URL, keywords, and categories, but the missing license and MSRV/CI policy are deliberate release blockers. The private fixture must remain local and ignored in every future validation workflow.
