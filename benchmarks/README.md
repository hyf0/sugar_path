# Performance baselines

SugarPath treats performance as three related but different questions. Criterion measures wall time on the current machine, CodSpeed tracks CPU and memory behavior in a controlled environment, and the allocation task records exact allocator calls plus requested bytes. No one number substitutes for the others.

## Workloads

Rolldown is the primary consumer, so the suite includes path lengths, component depths, and composed outputs shaped like its call sites. It also keeps short paths, dirty paths, leading parents, current-directory spellings, invalid native encoding, and Windows root forms visible so a fast common case cannot hide a regression elsewhere.

Every timed benchmark black-boxes both input and output. Setup that is not part of the consumer operation stays outside the measured closure. A benchmark that intentionally measures a batch declares byte or element throughput. The timed binaries use mimalloc 0.1.64 to match Rolldown.

Benchmark and allocation scenario names describe inputs and requested output shapes. Keep those identities unchanged when an implementation starts borrowing, consumes an owned buffer, or removes an intermediate value. Implementation-specific alternatives may use separate control names, but the public-operation row must remain stable so CodSpeed and committed allocation snapshots can compare the baseline with a later optimization.

In paired timing rows, `borrowed_receiver` means the operation receives a borrowed path view, while `owned_receiver` reserves the same final-output contract for an operation that may consume a prepared `PathBuf`. `natural_result` means the method's direct public return value; `pathbuf_result`, `string_result`, and the ArcStr slash labels name the requested intermediate or final container explicitly. When v2 has no consuming method, its owned-receiver control deliberately repeats the borrowed implementation so a later consuming implementation can retain the ID and change only the ownership mechanism.

Rows whose public path spelling or root semantics intentionally change in the breaking API are contract coverage, not same-output speed comparisons. In particular, do not use the trailing-separator or dot-separator normalization rows, or the Windows verbatim-UNC different-share relative row, to claim an algorithmic speedup against this baseline: the later API returns a deliberately different value in those cases. Keep them in the suite so the cost of the selected contract remains visible, and base same-output performance claims on rows whose exact result is unchanged.

## Commands

Run the full local timing suite in both the public default configuration and the `cached_current_dir` configuration:

```sh
cargo bench --locked
cargo bench --locked --features cached_current_dir
```

Save a local Criterion baseline before an experiment, then compare the experiment against it:

```sh
cargo bench --locked --bench absolutize --bench normalize --bench relative --bench to_slash --bench as_path --bench rolldown -- --save-baseline before-default
cargo bench --locked --bench absolutize --bench normalize --bench relative --bench to_slash --bench as_path --bench rolldown -- --baseline before-default
cargo bench --locked --features cached_current_dir --bench absolutize --bench normalize --bench relative --bench to_slash --bench as_path --bench rolldown -- --save-baseline before-rolldown
cargo bench --locked --features cached_current_dir --bench absolutize --bench normalize --bench relative --bench to_slash --bench as_path --bench rolldown -- --baseline before-rolldown
```

The explicit `--bench` selectors are required when forwarding Criterion-only arguments; otherwise Cargo also passes them to the library test harness. Use `cargo bench --locked --bench rolldown -- --quick` for a smoke test.

Criterion data lives under `target/criterion` and is not committed because wall time is machine-specific. Record the exact baseline commit, `rustc -Vv`, target, allocator, and command when reporting a result.

Generate or verify allocation snapshots for the current native target:

```sh
cargo allocs --write benchmarks/allocations/$(rustc -vV | sed -n 's|host: ||p')-default.snap
cargo allocs-rolldown --write benchmarks/allocations/$(rustc -vV | sed -n 's|host: ||p')-rolldown.snap
cargo allocs --check benchmarks/allocations/$(rustc -vV | sed -n 's|host: ||p')-default.snap
cargo allocs-rolldown --check benchmarks/allocations/$(rustc -vV | sed -n 's|host: ||p')-rolldown.snap
```

Allocation counts and reallocations are the cross-run gate. Requested bytes remain target-, feature-, and current-directory-shape-specific evidence rather than a portable invariant.

CodSpeed runs the same Criterion suite on Linux in simulation and memory modes. Native GitHub-hosted Linux and Windows runners check their allocation snapshots; macOS ARM64 evidence is recorded from a native Apple Silicon host. The Docker/Wine Windows-GNU instructions in [Windows GNU execution](./windows-gnu.md) are optional reproduction material, not part of normal development or CI. Do not run them unless the document's explicit local-execution gate is satisfied.

## Baseline rule

An accepted baseline contains the workload definitions, stable benchmark identities, pinned toolchain and dependency lock, native allocation snapshots, and green checks. Merge and run that baseline on the default branch before evaluating an optimization PR. Later performance changes keep the comparable identities intact and report allocation data plus CodSpeed or same-machine Criterion results against that accepted baseline.
