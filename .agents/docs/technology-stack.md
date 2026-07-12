# Technology stack

## Pinned stable Rust and standard path semantics

The repository pins Rust 1.97.0 in [`rust-toolchain.toml`](../../rust-toolchain.toml), and CI installs the same version explicitly. The root [`Cargo.lock`](../../Cargo.lock) is committed so benchmark and allocation baselines do not silently change their dependency graph. Update either pin deliberately and regenerate the performance evidence before comparing results across the change.

`std::path` is the semantic base: platform differences are inherited from the target and handled through compile-time branches, while the crate adds convenience and optimized lexical operations rather than replacing the standard path model.

Rust edition 2024 and dependency requirements remain machine-readable in [`Cargo.toml`](../../Cargo.toml); this record does not duplicate them.

## Performance dependencies

`memchr` is used where path work can skip directly between separator bytes. It earns a dependency by avoiding byte-by-byte scanning and unnecessary current-directory or `PathBuf` work on common absolute UTF-8 inputs. Commit [`94aca38`](https://github.com/hyf0/sugar_path/commit/94aca38606796098ed483a42dc74f8404186c256) records the measured reason for the relative-path fast path; keep performance claims tied to reproducible benchmarks rather than assuming every `memchr` use is faster.

`SmallVec` keeps the usual shallow component lists inline while allowing deep paths to spill to the heap without changing semantics. The inline capacity of eight is a workload bet, not a limit. Its current uses are the dot-normalizing relative-path slow path and Windows drive-relative absolutization; a clean deep path can bypass it entirely. Reconsider the capacity only with benchmark evidence, and preserve an execution-path-specific spillover case in [`tests/deep_paths.rs`](../../tests/deep_paths.rs).

## Benchmarks guard user-facing properties

Allocation avoidance and separator scanning are part of the library's stated value. Criterion2 measures named wall-time workloads with mimalloc 0.1.64, matching Rolldown's allocator. CodSpeed runs the same binaries in CPU-simulation and memory modes. The workspace allocation task records exact allocator-call snapshots plus target-specific requested bytes. Performance changes should compare the affected named workload rather than cite a single aggregate number.

`autobenches = false` is intentional. Each benchmark binary is declared explicitly in [`Cargo.toml`](../../Cargo.toml), while shared workloads live under `benches/support/` and cannot be mistaken for runnable targets. Commit [`94aca38`](https://github.com/hyf0/sugar_path/commit/94aca38606796098ed483a42dc74f8404186c256) preserves the original CodSpeed auto-discovery failure and rationale.

## Durable evidence

- [Manifest and explicit benchmark targets](../../Cargo.toml)
- [Benchmark methodology and commands](../../benchmarks/README.md)
- [Allocation tracker](../../tasks/track_allocations/README.md)
- [CodSpeed workflow](../../.github/workflows/codspeed.yml)
- [Normalization workloads](../../benches/normalize.rs)
- [Relative-path workloads](../../benches/relative.rs)
