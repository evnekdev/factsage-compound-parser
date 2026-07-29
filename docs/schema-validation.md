# Kaitai schema validation

Validation date: 2026-07-30

The private local fixture examples/MS16BASE.CDB remains ignored by Git. It was read in place and was not copied, staged, or included in generated output or documentation.

## Toolchain

- Kaitai Struct compiler: 0.11.0, official JavaScript compiler package.
- Kaitai Python runtime: 0.11.
- Python used for validation: 3.11.7.
- The JVM CLI was not installed on the validation machine. The official 0.11 JavaScript compiler API produced the same Python-target parser requested by this validation.
- The generated parser was written only to target/kaitai-python, which is ignored.

The schema metadata keeps Kaitai language version 0.10, quoted as a string. This is required because YAML otherwise parses 0.10 as numeric 0.1.

## Reproduction

If a Kaitai CLI is installed:

~~~text
New-Item -ItemType Directory -Force target/kaitai-python
kaitai-struct-compiler --target python --outdir target/kaitai-python schemas/factsage_compound.ksy
python scripts/validate_cdb_schema.py examples/MS16BASE.CDB
~~~

The exact fallback used here, using the official 0.11 JavaScript compiler package in isolated temporary storage, is:

~~~text
npm install --prefix C:\tmp\factsage-kaitai-compiler kaitai-struct-compiler@0.11.0 js-yaml
New-Item -ItemType Directory -Force target/kaitai-python
node -e "const fs=require('fs'); const yaml=require('C:\\tmp\\factsage-kaitai-compiler\\node_modules\\js-yaml'); const c=require('C:\\tmp\\factsage-kaitai-compiler\\node_modules\\kaitai-struct-compiler'); const ksy=yaml.load(fs.readFileSync('schemas/factsage_compound.ksy','utf8')); c.compile('python',ksy,null,false).then(files=>{for(const [name,content] of Object.entries(files)){fs.writeFileSync('target/kaitai-python/'+name,content);}}).catch(e=>{console.error(e&&e.stack||e); process.exit(1);});"
python scripts/validate_cdb_schema.py examples/MS16BASE.CDB
~~~

The validator accepts an alternate generated-parser directory with:

~~~text
python scripts/validate_cdb_schema.py examples/MS16BASE.CDB --generated-dir path\to\generated
~~~

The private CDB should not be uploaded to the Kaitai Web IDE automatically. The schema is ready for the repository owner to load manually with the local sample for an optional compatibility check.

## File and chunk results

- File size: 755,712 bytes.
- Expected chunk count: 2,952.
- Parsed chunk count: 2,952.
- Root Kaitai stream: position 755,712 of size 755,712.
- First raw ID: 9.
- First raw magic at byte offsets 2 through 5: CMPD.
- Unknown-ID count: 0.
- Raw and parsed histograms matched exactly:
- Parsed chunk-ID sequence matched the raw ID at every logical index.

~~~text
ID 1: 537
ID 2: 1517
ID 5: 1
ID 7: 620
ID 8: 64
ID 9: 1
ID 10: 212
~~~

The fixture does not contain IDs 3, 4, 6, or 11, so those switch cases were compiler-checked and body-size-checked but not exercised by a record in this file.

## Body-size assertions

Every known body totals exactly 255 bytes:

~~~text
database_header_body: 255
compound_body: 255
phase_ordinary_body: 255
phase_transition_body: 255
cp_body: 255
comment_body: 255
kappa_body: 255
~~~

All parsed bodies retained a 255-byte raw body. No alignment drift was detected. The parsed body classes were:

~~~text
CompoundBody: 537
CpBody: 1518
PhaseOrdinaryBody: 620
PhaseTransitionBody: 64
DatabaseHeaderBody: 1
CommentBody: 212
~~~

## Ordering and plausibility

- The compound-group state machine passed with no ordering violations.
- Fixed-width ASCII fields checked: 2,833; ASCII decode failures: 0; non-ASCII bytes: 0.
- Extended comment fields checked with Windows-1252: 212; decode failures: 0; non-ASCII bytes: 1.
- Width/alignment string failures: 0.
- OLE timestamp values checked: 2,952; non-finite: 0; minimum 0.0; maximum 44,398.97602458333.
- Phase thermodynamic values checked: 1,368; non-finite: 0; minimum -26,217,079.92999999; maximum 450,000.0.
- CP and kappa range bounds checked: 3,036; non-finite: 0; minimum 1.0; maximum 10,000.0.
- CP ranges with t_min <= t_max: 1,518 of 1,518.
- CP coefficients checked: 12,144; non-finite: 0.
- CP powers checked: 12,144; non-finite: 0.
- Signed phase-ID values checked: 2,886; outside signed 32-bit range: 0; minimum -991; maximum 991.
- Kappa coefficient/power values checked: 0 because this fixture contains no ID-11 records.
- Named padding, reserved, and unknown fields were retained; raw-body retention covered 752,760 bytes.

The validator reports aggregate values only and does not print compound names, formulae, comments, coefficients, or record bodies.

## Schema changes

1. Quoted meta ks-version 0.10 so the Kaitai compiler receives the intended version string instead of YAML numeric 0.1.
2. Changed comment_body.comment from ASCII to Windows-1252. One local comment fragment contains byte 0xB0; treating it as ASCII caused a generated-parser UnicodeDecodeError at chunk index 1,034. The revised encoding preserves all byte positions and decodes the complete fixture.
3. No field widths, switches, signedness, or array lengths required correction. All known body totals remain 255 bytes.

## Unresolved questions

- The fixture does not exercise IDs 3, 4, 6, or 11. A second private database containing those records should be used before implementing Rust semantics.
- The exact semantic distinction among CP IDs 2 through 6 remains undocumented.
- The comment text encoding should be confirmed against additional databases; this fixture supports Windows-1252.
- Kappa arrays and semantic physical-property evaluation remain unverified because no ID-11 record occurs here.
- OLE date interpretation and the meaning of preserved unknown/padding bytes are outside the structural Kaitai layer.

## Final assessment

**Structurally validated with warnings** - the complete private file parsed successfully, raw and parsed chunk counts and histograms matched, every parsed body retained exact 255-byte size, the root reached the exact file end, ordering passed, and representative values were finite and structurally plausible. Warnings remain for unexercised chunk variants and unresolved format semantics.
