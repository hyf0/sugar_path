# Project Context Records

Use this map to load only the context that bears on the work in front of you. All records are unstamped drafts until a human explicitly vouches them.

## Direction and public contract

- [Project goal and non-goals](./goal.md) — the intended library shape, the performance promise, and the boundary between lexical path work and filesystem resolution.
- [Public API redesign](./api-redesign.md) — the settled decision, implementation, and native allocation evidence for the breaking surface.
- [Semantic test strategy](./testing-strategy.md) — the finite behavior partitions, branch-entry requirements, native CI guarantees, and audited coverage gaps.
- [Public API and usage](../../README.md) — the supported operations, examples, features, and platform support exposed to users.

## Code routes

- `src/sugar_path.rs` or public API changes → [Public API redesign](./api-redesign.md), [Public surface and implementation boundary](./architecture.md#public-surface-and-implementation-boundary), and [The receiver is the target in `relative`](./gotchas.md#the-receiver-is-the-target-in-relative).
- `src/sugar_path_buf.rs` or consuming API changes → [Public API redesign](./api-redesign.md), [Extension traits over standard types](./architecture.md#extension-traits-over-standard-types), [Allocation is part of the contract](./architecture.md#allocation-is-part-of-the-contract), and [Historical consuming and fused API measurements](./performance-strategy.md#recorded-superseded-consuming-and-fused-api-experiment).
- `src/impl_sugar_path.rs` normalization or relative-path changes → [Execution paths](./architecture.md#execution-paths), [Exact path spelling is observable](./conventions.md#exact-path-spelling-is-observable), and [Normalization is lexical](./gotchas.md#normalization-is-lexical).
- `src/impl_sugar_path.rs` allocation or lifetime changes → [Allocation is part of the contract](./architecture.md#allocation-is-part-of-the-contract), [Allocation behavior needs explicit coverage](./conventions.md#allocation-behavior-needs-explicit-coverage), [`relative` borrows only from its receiver](./gotchas.md#relative-borrows-only-from-its-receiver), and [Explicit cwd never lends its lifetime to the output](./gotchas.md#explicit-cwd-never-lends-its-lifetime-to-the-output).
- `src/impl_sugar_path.rs` slash-conversion changes → [Slash conversion policy is explicit and does not normalize](./gotchas.md#slash-conversion-policy-is-explicit-and-does-not-normalize), [Encoding policy needs explicit coverage](./conventions.md#encoding-policy-needs-explicit-coverage), and [Exact path spelling is observable](./conventions.md#exact-path-spelling-is-observable).
- `src/utils.rs` or `cached_current_dir` → [Current-directory access is isolated](./architecture.md#current-directory-access-is-isolated) and [The cached current directory is process-lifetime state](./gotchas.md#the-cached-current-directory-is-process-lifetime-state).
- Windows prefixes, separators, drive-relative paths, UNC paths, or cross-platform behavior → [Platform behavior stays behind compile-time branches](./architecture.md#platform-behavior-stays-behind-compile-time-branches), [Platform-specific cases stay platform-gated](./conventions.md#platform-specific-cases-stay-platform-gated), and [Windows roots, drives, and namespaces carry different context](./gotchas.md#windows-roots-drives-and-namespaces-carry-different-context).

## Tooling and workflow routes

- Dependency or fast-path changes → [Performance dependencies](./technology-stack.md#performance-dependencies).
- Rust edition, toolchain policy, or the CI operating-system matrix → [Pinned stable Rust and standard path semantics](./technology-stack.md#pinned-stable-rust-and-standard-path-semantics) and [Platform behavior stays behind compile-time branches](./architecture.md#platform-behavior-stays-behind-compile-time-branches).
- Benchmarks, `Cargo.toml` benchmark targets, or CodSpeed → [Benchmarks guard user-facing properties](./technology-stack.md#benchmarks-guard-user-facing-properties).
- Performance optimization scope or a non-UTF-8 performance tradeoff → [UTF-8 is the default performance target](./performance-strategy.md#utf-8-is-the-default-performance-target).
- Historical Windows-GNU Docker/Wine reproduction → [Local Docker validation is opt-in](./conventions.md#local-docker-validation-is-opt-in) and [Windows GNU local execution gate](../../benchmarks/windows-gnu.md#local-execution-gate).
- Performance strategy, baseline changes, Rolldown workloads, allocation work, or API performance → [Performance strategy](./performance-strategy.md).
- Test fixtures, assertions, generated cases, or platform test coverage → [Testing conventions](./conventions.md) and [Semantic test strategy](./testing-strategy.md).
- Release-plz, crates.io trusted publishing, or post-merge test sequencing → [Tested release workflow](./release.md).
