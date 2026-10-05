# Bounded FDB construction evidence

Status: NATIVE EVIDENCE REGISTRY.

This record consolidates native FDB construction observations that were previously recorded downstream in `database-compare`. It does not by itself claim that the current crate implements every constructor mentioned in those historical experiments.

## Fresh Modern FDB profile

Evidence from controlled generated files and FactSage 7.3 acceptance established a bounded native profile:

- the generated FDB preserves a complete caller-supplied/versioned ID-9 header template rather than inventing unknown header bytes;
- formula construction uses the established native element/coefficient slots;
- native formula charge encoding used by the admitted generated profile is `stored = semantic_charge + 50`;
- generated formula element ordering that includes hydrogen must place hydrogen last; a hydrogen-first generated group failed to appear as a visible Function while the corresponding hydrogen-last group did;
- Function records precede their CP records in the native stream;
- generated files are reparsed and structurally verified before acceptance;
- controlled one-function, hydrogen-only, multiple-function, and charged synthetic Functions were accepted/displayed in FactSage 7.3 within the admitted profile.

These are bounded construction facts, not proof of every valid FDB shape.

## CP-kind construction evidence

Downstream paired/generated evidence established the following bounded construction behavior:

- ID-2 is used by the admitted generated nonzero-Cp ordinary Function profile;
- ID-5 is an evidenced exact-zero-Cp representation in translated/generated Function material;
- a translated corpus contains at least one Function combining ID-2 and ID-5 ranges;
- generated ID-2 and ID-5 probes displayed their Function in FactSage 7.3;
- this evidence does **not** establish a universal semantic meaning for all native CP IDs 2–6.

Therefore `CF-CPKIND-01` remains PARTIAL: construction may use these evidenced shapes in the bounded profile, while broader CP-kind semantics remain unresolved.

## Zero-object and opaque-field policy

- Explicitly zero optional added objects may be omitted only where the admitted construction profile has direct evidence for that omission.
- Unknown/opaque native fields must come from an admitted template or be preserved from source authority; they must not be semantically invented.
- Native acceptance of one template/profile does not establish a universal default for every CDB/FDB database.

## Verification boundary

A generated artifact is not accepted merely because it serializes. The evidence profile requires strict reparse/index validation and scientific checks over the fields whose semantics are established. Native FactSage open/save/reopen evidence is stronger than parser-only structural acceptance and should remain separately recorded.

## Source provenance

These bounded observations were migrated from `database-compare` architecture/evidence ledgers so native CDB/FDB knowledge has one authoritative home. Downstream Database Compare documents may retain historical experiment results, but current native claims should reference this file.