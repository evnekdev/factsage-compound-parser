# Performance measurements

## Method

Measurements use release builds on the local development machine. They are comparative observations, not hardware-independent promises. Private-fixture output is aggregate-only; Criterion benchmarks use generated valid streams and never use proprietary bytes.

`RawChunk` is 264 bytes on this target. Every typed known raw record measured 256 bytes: compound, CP, ordinary phase, transition phase, comment, and kappa. The four-byte enum tag/alignment overhead explains the raw chunk size.

## Baseline before indexed views

The 755,712-byte private fixture (2,952 chunks) measured over 12 release iterations:

| Operation | Median-like average |
| --- | ---: |
| Raw parse from bytes | 340 us |
| Semantic grouping with cloned records | 887 us |
| Serialization | 199 us |
| Editor domain rebuild | 910 us |

The owned raw vector capacity accounted for approximately 779,328 bytes. The old semantic representation held approximately 755,456 bytes of additional complete raw records. These are capacity/type-size estimates, not allocator measurements.

## Indexed-view results

The new `DomainIndex` stores only indexes, relationship vectors, and diagnostics. It contains no raw records, so the previous approximately one-file-sized semantic duplication has been removed. The raw vector remains contiguous and authoritative.

The same 755,712-byte private fixture measured over 12 release iterations after the refactor: raw parse 347 us, domain-index build 460 us, serialization 194 us, and editor index rebuild 483 us. The index's raw-record duplication is exactly zero by design; its small index/vector allocations are not allocator-profile measurements.

Short Criterion run (10 samples, 0.1 s warm-up and measurement) on generated valid streams:

| Operation | 256 KB | 1 MB | 5 MB | 7 MB |
| --- | ---: | ---: | ---: | ---: |
| `from_bytes` | 79.1 us | 1.13 ms | 6.49 ms | 8.33 ms |
| `from_reader` | 112 us | 1.57 ms | 11.56 ms | 15.99 ms |
| Domain index build | 97.5 us | 1.25 ms | 8.25 ms | 11.69 ms |
| View traversal | 609 ns | 7.37 us | 40.76 us | 70.87 us |
| `to_bytes` | 50.95 us | 575.87 us | 2.57 ms | 3.60 ms |
| `write_to` memory sink | 52.94 us | 632.60 us | 2.39 ms | 3.24 ms |
| Middle insertion | 95.20 us | 1.07 ms | 5.40 ms | 7.61 ms |
| Editor index rebuild | 234.20 us | 1.33 ms | 7.37 ms | 10.03 ms |

The reader benchmark uses a `Cursor`, not filesystem I/O. Streaming avoids a second retained full-file input buffer, but it is not expected to beat direct slice parsing. The short benchmark is intentionally useful for regression comparison only; repeated runs showed normal local-machine variance.

## Interpretation

`Vec<RawChunk>` is appropriate for the target file sizes. Sequential parsing, indexing, traversal, and serialization are fast enough that a file-backed zero-copy layer is not justified by the measured memory or timing behavior. Middle insertion is intentionally linear; CDB edits are expected to be infrequent relative to reads.

Run the synthetic benchmark with:

```text
cargo bench --bench architecture
```
## Windows 0.1.0 publication validation

The following release measurements were taken on Windows 11 Enterprise build 26200, with Rust 1.97.1 (`x86_64-pc-windows-msvc`), a 13th Gen Intel Core i7-13850HX, and 31.7 GiB visible memory. They use release builds, are aggregate-only, and should be read as local observations rather than guarantees.

The ignored 755,712-byte local fixture (2,952 chunks), measured over 12 iterations, produced:

| Operation | Average |
| --- | ---: |
| `from_bytes` | 487 us |
| `from_reader` | 763 us |
| `from_path` | 4,991 us |
| Domain-index build | 301 us |
| View traversal | 2 us |
| `to_bytes` | 201 us |
| Streaming `write_to` to memory | 241 us |
| Middle insertion plus raw clone | 418 us |
| Editor index rebuild after structural edit | 425 us |

The read-only installation-corpus validator processed 12 valid Compound Databases (15,165,184 bytes and 59,239 chunks). Aggregate release-mode elapsed times were 91.536 ms for `from_path`, 6.054 ms for `from_bytes`, 12.623 ms for `from_reader`, 6.092 ms for index construction, 6.442 ms for deterministic view traversal, 6.210 ms for `to_bytes`, and 10.532 ms for streaming serialization. Filesystem caching, antivirus, storage latency, and CPU frequency materially affect these numbers, especially `from_path`.

The corpus confirms that a contiguous owned `Vec<RawChunk>` is suitable for the expected 5-7 MB CDB files. It does not justify a file-backed zero-copy or memory-mapped representation: the current model avoids semantic raw-record duplication while retaining straightforward ownership and predictable streaming I/O.