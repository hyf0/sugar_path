# Allocation snapshot task

This package records heap allocation behavior for individual SugarPath operations and for composed path pipelines. It is separate from the timing harness: timing answers how long an operation takes, while this task records which successful global-allocation entry points it uses.

Scenario names describe the input ownership and requested output shape rather than a concrete implementation method or return container. Keep the accepted baseline identities stable when an implementation starts borrowing, consumes an owned buffer, or removes an intermediate value. That makes the saved pre-optimization and optimized snapshots directly comparable.

Owned-input normalization, clean and dirty joins, valid slash conversion, descendant and upward relative output, normalized String output, and ArcStr output have paired rows. One row keeps the borrowed receiver or natural result; the other requests an owned receiver or PathBuf result. SugarPath v2 intentionally executed several pairs through the same operation. The v3 bodies preserve the names while using the consuming API where ownership makes buffer reuse possible.

The clean allocation targets have dedicated rows rather than being inferred from a larger pipeline. A canonical native descendant returned naturally performs zero allocations; requesting a PathBuf performs exactly its one final output allocation. Descendant and upward owned-result String compositions each perform exactly one allocation for the reusable final buffer. Clean owned-input normalization and valid-Unicode owned-input slash conversion perform zero allocations. The clean owned-cwd absolutize row excludes cwd construction and distinguishes a buffer-growth `realloc` from a fresh allocation.

`CountingAllocator` wraps `mimalloc_safe::MiMalloc` and records successful `alloc`, `alloc_zeroed`, and `realloc` calls. It also records the sizes requested by `alloc` and `alloc_zeroed`, and the new sizes requested by `realloc`. Deallocations are not counted because the optimization target is avoiding allocation work, not retaining memory.

Each scenario has separate setup and operation phases. Allocation-capable input setup runs with tracking disabled before the warm-up operation and before every measured operation; this excludes construction of invalid-encoding paths, owned cwd values, and owned clean inputs. Operations named `join` include the join allocation by design. Every scenario is warmed once and then measured seven times in a single-threaded process, and the executable fails if any sample differs. Result formatting, snapshot parsing, and file I/O stay outside tracked regions; `std::hint::black_box` keeps inputs and results observable to the optimizer.

The scenario matrix covers clean and dirty normalization, the 24/25-component owned-reuse boundary, current-directory spellings, leading-parent paths, invalid native encoding, absolute and relative path relations, exact-capacity as well as ordinary borrowed and owned cwd inputs, native slash conversion, and composed PathBuf, String, and ArcStr outputs. The sideEffects rows compare the public relative operation with a caller-side strip-prefix fallback for both descendant hits and upward misses; their temporary text borrows from the path result. Windows builds add the 64/65-component and 512-byte replay/spill boundaries, mixed separators, drive-relative and root-relative paths, ordinary and verbatim UNC roots, long-prefix consuming normalization, and invalid wide strings.

## Run

Run from the repository root with the same Rust toolchain, target, build profile, and features whenever results are compared:

```sh
# Continuous CI gates (run on a matching host)
cargo allocs --check benchmarks/allocations/x86_64-unknown-linux-gnu.snap
cargo allocs --check benchmarks/allocations/x86_64-pc-windows-msvc.snap

# Local print for the current host
cargo allocs

# Rewrite a committed gate only on the matching native host or via workflow_dispatch
cargo allocs --write benchmarks/allocations/x86_64-unknown-linux-gnu.snap
cargo allocs --write benchmarks/allocations/x86_64-pc-windows-msvc.snap
```

`cargo allocs` is the only entry point. The tracker always builds SugarPath with `cached_current_dir`, which is the production configuration this gate protects. The default mode prints a generated Markdown snapshot. `--write PATH` replaces the named file. `--check PATH` requires the saved platform, target environment, profile, configuration, numeric current-directory shape, and allocation-call table to match; it exits successfully with an informational message when only the requested-byte table changes. Snapshot parsing is line-ending independent, so native Windows CRLF files and LF-normalized checkouts use the same hard-gate table.

Only two snapshots are committed and checked: Linux x86_64 GNU and Windows x86_64 MSVC, on native GitHub-hosted runners with Rust 1.97.0. Prints on other hosts stay local diagnostics. [`benchmarks/windows-gnu.md`](../../benchmarks/windows-gnu.md) preserves an optional Docker/Wine reproduction procedure; it is not part of the default local workflow and its GNU output must never be relabeled as MSVC evidence.

## Interpreting the snapshot

Allocation call counts are the hard regression gate because they directly represent how often code crosses the allocator boundary, and they remain meaningful across many implementation changes. Requested bytes are supporting evidence: `PathBuf`, `String`, operating-system path representations, the standard library, the target architecture, and compiler optimization can change requested capacities without changing the number of allocation operations. Keep one baseline per platform and treat byte changes as an investigation prompt rather than a portable pass/fail threshold.

Current-directory lookup affects both call counts and requested capacities. On Unix the task enters the deterministic writable directory `/tmp/sugar_path_track_allocations/cwd` before warming any SugarPath call. On Windows it uses a same-named child of the system temporary directory because no fixed writable drive is portable. The snapshot records only the resulting encoded byte length and component count, never the original or measurement directory text; this makes environmental differences visible without exposing a personal path. The original current directory is restored before output or file I/O.

This task counts only allocations routed through Rust's global allocator on the current process. It does not measure stack use, allocator-internal virtual-memory work, peak live memory, or allocations in another process. Run it without unrelated worker threads because the tracking switch is process-wide.
