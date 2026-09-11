# 0.1.0 release-readiness review

## Recommendation

**Ready to publish experimental 0.1.0.**

This recommendation is for an experimental library release only. It does not claim that all reverse-engineered FactSage semantics are production-ready.

The project is independent and reverse-engineered. It is not affiliated with,
endorsed by, or supported by the FactSage developers or distributors. FactSage
is a trademark of its respective owners.

## Windows-first validation

Windows is the primary validated platform because FactSage and the reference databases are Windows-based. The implementation itself uses portable Rust interfaces where practical; Linux and macOS CI remain secondary portability checks.

On Windows 11 Enterprise build 26200, x86_64 MSVC Rust 1.97.1, with a 13th Gen Intel Core i7-13850HX and 31.7 GiB visible memory, the Windows corpus validator scanned 13 `.CDB` candidates totaling 15,345,664 bytes.

- 12 files (15,165,184 bytes and 59,239 chunks) were valid Compound Databases.
- All 12 parsed from path, bytes, and reader equivalently.
- All 12 built deterministic indexes, rejected index reuse with a cloned raw stream, and serialized byte-for-byte in memory.
- There were zero domain failures, diagnostics, unknown IDs, orphan ranges, or ambiguous phase links in valid databases.
- One `.CDB` candidate began with ID 0 rather than the required ID-9 Compound Database header. It is reported as an unsupported format candidate; no format meaning is inferred.

## ID-11 structural evidence

Three valid databases contained 86 ID-11 records. All 86 records round-tripped exactly and had finite parsed numeric fields plus non-reversed temperature bounds.

- 53 compounds and 86 phases contained linked ID-11 records.
- All receiving phases were solid, with observed raw IDs from 101 through 108.
- There were zero orphan or ambiguous ID-11 links.
- All ID-11 records followed CP/range records; 52 preceded another CP record and 5 preceded a comment fragment. None followed a comment fragment.
- Synthetic tests verify multiple linked records preserve stream order, ambiguous ID-11 links remain orphaned, unrelated insertion/removal rebuilds correct links, and non-structural edits leave ID-11 bytes untouched.

This validates structure and linking, not a physical kappa equation, units, or coefficient semantics.

## Completed release foundation

- native lossless parser and serializer with float-bit preservation;
- one authoritative `RawDatabase`, index-only `DomainIndex`, borrowed views, and lazy raw-authoritative editor;
- Windows stable quality CI and Windows 1.85.0 MSRV validation;
- typed open/read/create/write path errors retaining OS error sources;
- synthetic I/O-fault coverage for failures before and between chunks, partial output, invalid paths, directories, spaces, Unicode names, and supported long paths;
- synthetic regression coverage for every known ID, ID-11 linking/preservation, corruption handling, and editor locality;
- property tests, bounded fuzz targets, strict public Rustdoc, Dependabot, dependency review, and `cargo audit`;
- package-list and clean package verification.

## Classification of remaining issues

### Publication blockers

None for an explicitly experimental 0.1.0 release. The package is not published by this task.

### Production-readiness blockers

- ID-11 physical equations, units, and coefficient meanings remain unverified.
- Magnetic, pressure-volume, transition, and ID-11 Gibbs contributions remain unsupported and typed as blockers or pending evidence.
- Density unit and high-order encoding remain unverified.
- Unknown/reserved field meanings remain unverified.
- Sustained sanitizer-backed fuzzing and a broader malformed external-file corpus remain desirable.
- The public API needs real downstream feedback before a stable 1.0 commitment.

### Documented limitations

- Only Compound Database streams beginning with ID 9 and `CMPD` are accepted.
- A `.CDB` extension alone does not establish this format; the observed ID-0 candidate is intentionally rejected.
- Writes are caller-directed and can leave a partial destination after an operating-system write failure.
- Corpus validation is local, ignored, read-only, and aggregate-only.

### Future enhancements

- More adversarial corpus collection and long Windows fuzz runs;
- validated kappa semantics if independent evidence becomes available;
- broader performance tracking across Windows machines;
- API-stability review before a non-experimental release.

## Packaging policy

The crate uses Cargo's default package inclusion rules. The package includes tracked source, tests, documentation, schemas, license files, and examples. It excludes `target`, fuzz build output, generated fuzz lockfiles, corpus reports, and ignored proprietary databases. `Cargo.lock` remains tracked for reproducible development and CI, while library consumers resolve dependencies normally.
