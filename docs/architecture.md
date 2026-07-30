# Ownership, indexing, and editing architecture

## Authoritative representation

`RawDatabase` owns a contiguous `Vec<RawChunk>`. It is the only authoritative physical representation and the only source used for serialization. Each `RawChunk` preserves its original ID, typed fields, fixed-width text bytes, padding, reserved bytes, unknown bytes, and CP ID variant.

The format uses 256-byte chunks and expected database sizes are approximately 5–7 MB. A contiguous vector gives predictable sequential traversal and serialization, good cache locality, and straightforward `O(n)` insertion/removal. Measurements do not justify a file-backed parser, memory mapping, a piece table, or linked-list storage.

`RawDatabase::from_reader` reads one exact record at a time, avoiding a second full-file input buffer. `RawDatabase::write_to` similarly writes one chunk at a time. `to_bytes` remains a convenience allocation for callers that need a single output vector.

## Semantic index and borrowed views

`DomainIndex` records only physical chunk indexes, phase/range links, orphan range indexes, unknown chunk indexes, and diagnostics. It does not own any `RawChunk` or a second raw record stream.

`DatabaseView<'a>` borrows both `&'a RawDatabase` and `&'a DomainIndex`. `CompoundView`, `PhaseView`, and range views borrow the raw records selected by the index. Normal semantic iteration does not allocate or clone raw chunks.

The index stores the raw stream identity and structural generation captured at build time. `DatabaseView::new` rejects an index built for another stream or an earlier structural generation. This prevents accidental use of stale indexes without self-referential structs or unsafe code.

## Index invalidation policy

Low-level `RawDatabase::chunks_mut`, `insert_chunk`, `push_chunk`, and `remove_chunk` permit arbitrary physical streams. They explicitly invalidate all indexes. Semantic validation happens when `DomainIndex::build` or `DatabaseEditor::view` rebuilds an index; invalid known ordering is then a fatal `DomainError`.

`DatabaseEditor` owns one `RawDatabase` and an optional lazy `DomainIndex`.

- Structural edits invalidate the index: insertion, removal, reordering through raw mutable access, changing a chunk variant/ID, or changing relationship IDs through raw access.
- Supported non-structural setters retain the index: compound/phase names, ordinary enthalpy/entropy, transition enthalpy/temperature, and real stoichiometric coefficients.
- `DatabaseEditor::view` rebuilds missing or stale indexes deterministically. A rebuild error leaves raw data untouched and no current index cached.

Borrowed views cannot survive a mutable editor borrow, so stale references are prevented by Rust's borrowing rules.

## Migration from the pre-index API

Earlier `0.1.0` releases exposed a grouped `Database` containing cloned raw compound, phase, and range records through public vectors. That API duplicated almost the full database and made editor rebuilds clone the raw stream.

Use a view instead:

```rust
use factsage_compound_parser::domain::Database;

fn count_phases(bytes: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
    let database = Database::from_bytes(bytes)?;
    let view = database.view()?;
    Ok(view
        .compounds()
        .map(|compound| compound.phases().count())
        .sum())
}
```

Raw records remain available through `compound.raw()`, `phase.raw()`, `range.raw()`, and `raw.chunks()`. Public raw chunk storage is now private; use `RawDatabase::chunks()` for immutable inspection and the documented structural methods for mutation.

## Preserved behavior

The forward grouping state machine remains `database header → compound → phases → ranges → comments → next compound or EOF`. Range links use exact `phase_id_raw` equality inside the current compound. Duplicate IDs leave ranges orphaned as ambiguous; missing IDs leave ranges orphaned as missing. Unknown chunks inside a compound are retained and diagnosed. Index construction never rewrites, reorders, or drops raw chunks.
