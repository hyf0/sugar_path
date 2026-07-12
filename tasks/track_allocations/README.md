# Allocation snapshot task

This package records heap allocation behavior for individual SugarPath operations and for composed path pipelines. It is separate from the timing harness: timing answers how long an operation takes, while this task records which successful global-allocation entry points it uses.

Scenario names describe the input and requested output shape rather than a concrete implementation method or return container. Keep those names stable when an implementation changes from an owned result to a borrowed result, or when a consuming API replaces an intermediate conversion. That makes the saved pre-optimization and optimized snapshots directly comparable.

`CountingAllocator` wraps `mimalloc_safe::MiMalloc` and records successful `alloc`, `alloc_zeroed`, and `realloc` calls. It also records the sizes requested by `alloc` and `alloc_zeroed`, and the new sizes requested by `realloc`. Deallocations are not counted because the optimization target is avoiding allocation work, not retaining memory.

Each scenario has separate setup and operation phases. Allocation-capable input setup runs with tracking disabled before the warm-up operation and before every measured operation; this excludes construction of invalid-encoding paths, owned cwd values, and owned clean inputs. Operations explicitly named `join` include the join allocation by design. Every scenario is warmed once and then measured seven times in a single-threaded process, and the executable fails if any sample differs. Result formatting, snapshot parsing, and file I/O stay outside tracked regions; `std::hint::black_box` keeps inputs and results observable to the optimizer.

The scenario matrix covers clean and dirty normalization, current-directory spellings, leading-parent paths, invalid native encoding, absolute and relative path relations, explicit borrowed and owned cwd inputs, native slash conversion, and composed PathBuf, String, and ArcStr outputs. Windows builds add mixed separators, drive-relative and root-relative paths, ordinary and verbatim UNC roots, and invalid wide strings.

Owned-input normalization, clean and dirty joins, valid slash conversion, descendant and upward relative output, normalized String output, and ArcStr output have paired rows. One row keeps the borrowed or natural-result route; the other requests an owned receiver or PathBuf result. SugarPath v2 has no consuming normalize or slash API and its natural relative result is already a PathBuf, so several baseline pairs intentionally execute the same v2 operation. The scenario names remain distinct and stable so the v3 implementation can replace only the owned route and expose its allocation change directly.

The sideEffects rows compare the public relative operation with a caller-side strip-prefix fallback for both descendant hits and upward misses. Their final temporary text borrows from the path result, so they isolate the result allocation rather than adding a String allocation.

## Run

Run from the repository root with the same Rust toolchain, target, build profile, and features whenever results are compared:

```sh
cargo allocs
cargo allocs --write benchmarks/allocations/$(rustc -vV | sed -n 's|host: ||p')-default.snap
cargo allocs-rolldown --write benchmarks/allocations/$(rustc -vV | sed -n 's|host: ||p')-rolldown.snap
cargo allocs --check benchmarks/allocations/$(rustc -vV | sed -n 's|host: ||p')-default.snap
cargo allocs-rolldown --check benchmarks/allocations/$(rustc -vV | sed -n 's|host: ||p')-rolldown.snap
```

The `allocs` alias measures SugarPath's public default feature set. `allocs-rolldown` enables `cached_current_dir`. The default mode prints a generated Markdown snapshot. `--write PATH` replaces the named file. `--check PATH` requires the saved platform, target environment, profile, configuration, numeric current-directory shape, and allocation-call table to match; it exits successfully with an informational message when only the requested-byte table changes. Snapshot parsing is line-ending independent, so native Windows CRLF files and LF-normalized checkouts use the same hard-gate table.

Linux and Windows snapshots are generated and checked on native GitHub-hosted runners. A macOS ARM64 snapshot can be generated on a native Apple Silicon host. The optional Windows-GNU Docker/Wine reproduction steps are documented in [Windows GNU execution](../../benchmarks/windows-gnu.md); they are never part of the default local workflow and must not be run without the explicit gate in that document.

## Interpreting snapshots

Allocation call counts are the hard regression gate because they directly represent how often code crosses the allocator boundary. Requested bytes are supporting evidence: path representation, standard-library behavior, target architecture, compiler optimization, and output capacity can change requested sizes without changing the number of allocation operations. Keep one baseline per platform and treat byte changes as an investigation prompt rather than a portable pass/fail threshold.

Current-directory lookup affects both call counts and requested capacities. On Unix the task enters the deterministic writable directory `/tmp/sugar_path_track_allocations/cwd` before warming any SugarPath call. On Windows it uses a same-named child of the system temporary directory because no fixed writable drive is portable. The snapshot records only the resulting encoded byte length and component count. The original current directory is restored before output or file I/O.

This task counts only allocations routed through Rust's global allocator in the current process. It does not measure stack use, allocator-internal virtual-memory work, peak live memory, or allocations in another process. Run it without unrelated worker threads because the tracking switch is process-wide.
