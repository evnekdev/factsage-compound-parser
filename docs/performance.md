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
| `from_bytes` | 74.4 us | 1.26 ms | 5.58 ms | 7.46 ms |
| `from_reader` | 99.6 us | 1.53 ms | 9.90 ms | 11.13 ms |
| Domain index build | 83.8 us | 1.18 ms | 7.73 ms | 11.65 ms |
| View traversal | 557 ns | 7.65 us | 47.2 us | 62.2 us |
| `to_bytes` | 24.2 us | 523 us | 2.56 ms | 4.79 ms |
| `write_to` memory sink | 48.9 us | 484 us | 2.32 ms | 4.05 ms |
| Middle insertion | 98.6 us | 1.03 ms | 4.68 ms | 7.70 ms |
| Editor index rebuild | 289 us | 1.37 ms | 10.44 ms | 11.13 ms |

The reader benchmark uses a `Cursor`, not filesystem I/O. Streaming avoids a second retained full-file input buffer, but it is not expected to beat direct slice parsing. The short benchmark is intentionally useful for regression comparison only; repeated runs showed normal local-machine variance.

## Interpretation

`Vec<RawChunk>` is appropriate for the target file sizes. Sequential parsing, indexing, traversal, and serialization are fast enough that a file-backed zero-copy layer is not justified by the measured memory or timing behavior. Middle insertion is intentionally linear; CDB edits are expected to be infrequent relative to reads.

Run the synthetic benchmark with:

```text
cargo bench --bench architecture
```