# Performance baselines

SugarPath treats performance as three related but different questions. Criterion measures wall time on the current machine, CodSpeed CPU simulation tracks instructions and cache-sensitive equivalent cycles, and the allocation task records exact allocator calls plus requested bytes. No one number substitutes for the others.

## Workloads

Rolldown is the primary consumer. The benchmark paths are public repository paths sampled from Rolldown commit `b9823050bc658ef65105148ea0504d4fbda7fa4c`. All 12,287 tracked repository-relative paths are included in the distribution: p50 is 75 bytes and 7 components, p90 is 102 bytes and 9 components, and p99 is 129 bytes and 10 components. The synthetic Unix absolute paths add the 20-byte `/workspace/rolldown/` prefix. Reproduce the numbers with `bash benchmarks/rolldown-path-distribution.sh /path/to/rolldown`. The suite separately names fast paths, slow paths, relative inputs, Windows roots, and composed Rolldown call patterns so one class cannot hide another's regression.

Every timed benchmark black-boxes both input and output. Setup that is not part of the consumer operation stays outside the measured closure. A benchmark that intentionally measures a batch declares byte or element throughput. Owned-output controls prepare their `PathBuf` outside the timed closure, while join and relative pipelines keep the work performed by the Rolldown-shaped caller inside it. The final-API matrix names the output shape explicitly: the main `relative -> Cow<Path>` result, `Cow::into_owned -> PathBuf`, borrowed strict slash conversion, and the ordinary strict consuming `relative(base).into_owned().into_slash() -> String` composition. Direct `Path` and `str` receiver rows keep receiver-specific cost visible. No current row represents a public fused relative-to-string API. Relative-output controls also include Rolldown's pinned `ArcStr` 1.2.0 final container because converting from `String` or `Cow<str>` performs another allocation and copy; a string-only result is not the end-to-end consumer cost for those call sites. The package-sideEffects controls compare the main Cow result with a `strip_prefix` plus relative-fallback control for both descendant and upward shapes. The pinned ThreeJS/Rome trace records 4,888 descendant hits and two upward misses for that exact caller, so its hit and miss costs must be combined at that caller-specific weight rather than judged from the zero-allocation hit alone.

Benchmark and allocation scenario names describe inputs and requested output shapes. Keep accepted identities unchanged when an implementation starts borrowing, consumes an owned buffer, or removes an intermediate value. Implementation-specific alternatives may use separate control names only when they measure additional work; do not duplicate an existing timed operation under a mechanism-specific name. Stable public-operation rows let CodSpeed and saved Criterion baselines compare the accepted baseline with later implementations.

In paired timing rows, `borrowed_receiver` means the operation receives a borrowed path view, while `owned_receiver` reserves the same final-output contract for an operation that may consume a prepared `PathBuf`. `natural_result` means the method's direct public return value; `pathbuf_result`, `string_result`, and the ArcStr slash labels name the requested intermediate or final container explicitly. The v2 baseline duplicates a borrowed implementation where it had no consuming method, allowing the final consuming implementation to retain the ID while changing only the ownership mechanism.

Rows whose public path spelling or root semantics intentionally change in the breaking API are contract coverage, not same-output speed comparisons. In particular, do not use the trailing-separator or dot-separator normalization rows, or the Windows verbatim-UNC different-share relative row, to claim an algorithmic speedup against the baseline: the final API deliberately returns a different value in those cases. Keep them in the suite so the cost of the selected contract remains visible, and base same-output performance claims on rows whose exact result is unchanged.

The benchmark binaries use mimalloc 0.1.64 because Rolldown uses the same allocator. Oxc's never-grow-in-place benchmark allocator was considered but is not the default here: it produces a stable worst-case reallocation cost, while the primary goal is to predict Rolldown. Use it only as a separate diagnostic if real allocator variance prevents a decision.

## Commands

Run the full local timing suite in both the public default configuration and Rolldown's `cached_current_dir` configuration:

```bash
cargo bench --locked
cargo bench --locked --features cached_current_dir
```

Save a local Criterion baseline before an experiment, then compare the experiment against it:

```bash
cargo bench --locked --bench absolutize --bench normalize --bench relative --bench to_slash --bench as_path --bench rolldown -- --save-baseline before-default
cargo bench --locked --bench absolutize --bench normalize --bench relative --bench to_slash --bench as_path --bench rolldown -- --baseline before-default
cargo bench --locked --features cached_current_dir --bench absolutize --bench normalize --bench relative --bench to_slash --bench as_path --bench rolldown -- --save-baseline before-rolldown
cargo bench --locked --features cached_current_dir --bench absolutize --bench normalize --bench relative --bench to_slash --bench as_path --bench rolldown -- --baseline before-rolldown
```

The explicit `--bench` selectors are required when forwarding Criterion-only arguments; otherwise Cargo also passes them to the library test harness, which rejects them. A full two-configuration run takes several minutes because every named case gets a three-second warm-up and roughly five seconds of sampling. Use `cargo bench --locked --bench rolldown -- --quick` for a smoke test.

Criterion data lives under `target/criterion` and is not committed because wall time is machine-specific. Record the exact baseline commit, `rustc -Vv`, target, allocator, and command when reporting a result.

The continuous allocation gate is intentionally small: two committed Rolldown (`cached_current_dir`) snapshots checked natively in CI.

```bash
# Continuous gates (must run on a matching host/target)
cargo allocs-rolldown --check benchmarks/allocations/x86_64-unknown-linux-gnu-rolldown.snap
cargo allocs-rolldown --check benchmarks/allocations/x86_64-pc-windows-msvc-rolldown.snap

# Local investigation on the current host (print; not a committed gate)
cargo allocs-rolldown
cargo allocs   # public default features, also local-only
```

Regenerate a gate snapshot only on the matching native host (or via the workflow_dispatch job), then commit the updated file:

```bash
cargo allocs-rolldown --write benchmarks/allocations/x86_64-unknown-linux-gnu-rolldown.snap
cargo allocs-rolldown --write benchmarks/allocations/x86_64-pc-windows-msvc-rolldown.snap
```

Allocation counts and reallocations are the cross-run gate. Requested bytes are recorded because they expose repeated short-lived buffers in this crate, but they remain target-, feature-, and current-directory-shape-specific evidence rather than a portable invariant.

CodSpeed runs the Rolldown configuration of the same Criterion suite in two modes on Linux: `simulation` records executed instructions, L1/last-level cache effects, equivalent cycles, and profiles; `memory` records allocator activity and peak heap behavior. Branch misses are not a continuous gate. Use Linux Callgrind with branch simulation only when a concrete branch-layout hypothesis needs diagnosis.

The committed final-API snapshots record zero allocation calls for canonical descendant `relative -> Cow<Path>`, one for `Cow::into_owned`, one for each descendant and upward strict final-`String` composition, zero for clean `PathBuf::into_normalized`, zero for valid-Unicode `PathBuf::into_slash`, and no fresh allocation plus one growth reallocation when a clean relative receiver consumes an owned cwd through `absolutize_with`. Requested bytes remain platform-specific.

Ordinary PR CI checks the two Rolldown snapshots on native Linux and Windows runners. Default-feature and macOS results are not continuous gates; print them locally when a regression is suspected. Native GitHub Actions generation run [`29181673809`](https://github.com/hyf0/sugar_path/actions/runs/29181673809) produced the Linux and Windows files for the final API. The older Windows-GNU snapshots were removed rather than relabeled; the pinned Docker/Wine commands in [Windows GNU execution](./windows-gnu.md) remain an opt-in historical reproduction reference. Native Windows timing is still required before making Windows-specific speed claims, while CodSpeed remains the continuous Linux CPU and memory view.

## Baseline rule

The first accepted baseline is a commit containing the workload definitions, toolchain and dependency lock, allocation snapshots, and green checks. Implementation optimizations start only after that commit. Each later performance commit names the affected workload and compares allocation data plus CodSpeed or same-machine Criterion results against the baseline or the immediately preceding accepted commit.
