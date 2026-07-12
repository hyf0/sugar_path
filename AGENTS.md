# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Test (all platforms run in CI: ubuntu, macos, windows)
cargo test --locked --workspace
cargo test --locked --workspace --all-features

# Single test
cargo test --test normalize           # run one test file
cargo test --test normalize unix      # run specific test fn

# Lint (CI runs with -D warnings — zero warnings allowed)
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings

# Documentation (CI treats rustdoc warnings as errors)
RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --all-features --no-deps

# Format (2-space indent, enforced in CI)
cargo fmt --all --check     # check only
cargo fmt --all             # fix

# Benchmarks (criterion2, custom harness)
cargo bench --locked --bench normalize
cargo bench --locked --bench absolutize -- absolutize_with
cargo bench --locked --features cached_current_dir  # match Rolldown

# Allocation baseline (CI gates: Linux + Windows snaps; always cached_current_dir)
cargo allocs --check benchmarks/allocations/x86_64-unknown-linux-gnu.snap   # on Linux CI host
cargo allocs --check benchmarks/allocations/x86_64-pc-windows-msvc.snap     # on Windows CI host
# Local print for the current host (not a continuous gate unless the target matches):
cargo allocs
```

## Architecture

Two extension traits over standard path types: `SugarPath` adds borrowed operations to `Path` and `str`/`String`; `SugarPathBuf` adds consuming operations that can reuse an owned `PathBuf`.

- **`README.md` / `src/lib.rs`** — Task-oriented user entry point and docs.rs crate landing page
- **`src/sugar_path.rs`** — Borrowed trait definition and authoritative method contracts
- **`src/sugar_path_buf.rs`** — Consuming `PathBuf` trait definition with doc examples
- **`src/impl_sugar_path.rs`** — All implementations. Two impl blocks: one for `Path`, one for `str`; `String` and other string-like values use normal deref method lookup. Contains `normalize_inner()`, `needs_normalization()`, `relative_str()` and helper functions
- **`src/utils.rs`** — `get_current_dir()` helper for `absolutize()`

Key patterns:
- `Cow<'_, Path>` return types to avoid allocation when the input is already clean
- `memchr`-accelerated scanning for separator-based fast paths
- `SmallVec` for stack-allocated component lists: 8 entries in normalization rebuilds and 16 in lexical relative calculation, based on the recorded Rolldown path-depth distribution
- Platform-specific code via `#[cfg(target_family = "unix")]` / `#[cfg(target_family = "windows")]` — dual implementations of `needs_normalization()` and Windows-specific logic in `relative()`, `normalize_inner()`

## Testing

- **`tests/test_utils.rs`** — Macros: `p!("path")` creates `&Path`, `assert_eq_str!(path_expr, "str")` compares via `.to_str().unwrap()`
- Tests are platform-gated with `#[cfg(target_family = "...")]`
- Benchmarks share named workloads from `benches/support/workloads.rs`
- The accepted pre-change performance baseline is main commit `9e6b627` from PR #41, whose tree matches reviewed baseline tip `e483f8f`; PR #40 follows that commit. Preserve neutral benchmark identities based on receiver/input shape and requested output shape, and do not relabel historical `9712b6e` or Windows-GNU measurements as results for the accepted baseline.
- The Docker/Wine commands in `benchmarks/windows-gnu.md` are an opt-in reproduction reference. Do not execute any local `docker` command unless a non-Docker check has already found an existing Docker installation and the developer or maintainer explicitly requested Docker execution in the current task. General requests for full validation, Windows coverage, or PR completion do not grant that permission. Never install or start Docker, pull the image, create volumes, or launch a container otherwise; rely on the CI operating-system matrix when the gate is not satisfied.

## Conventions

- `normalize()` and `normalize_inner()` return `Cow<'_, Path>` — borrowed when input needs no work, owned when normalization occurs
- Performance changes should compare the affected named timing workload and allocation snapshot against a committed baseline
- Windows paths use `\` as separator; forward slashes in input trigger normalization

<!-- PCR:START -->
## Project Context Records (PCR)

This project follows **Project Context Records (PCR)** — methodology: https://github.com/hyf0/project-context-records. PCR keeps the project's durable design context — the *why*, the decisions, the architecture — so you inherit it instead of re-deriving or re-litigating what's already settled.

When working here:
- **Where they live.** Records are in `.agents/docs/`, one topic per file, cross-linked with relative Markdown links. A `README.md` there is the **map**: it routes code areas or hotspots to the exact record or heading. Create one when retrieval stops being a glance or one record grows into a long ledger.
- **Read first.** Start from the map if present, else scan the folder. Open the exact records or headings that cover an area before changing or answering for it.
- **Use the strongest durable form.** Put machine-checkable constraints in types, tests, lints, or CI; put local rationale beside the code with a link; use PCR for cross-cutting judgment, intent, and other context that must remain prose.
- **Record as you go.** Capture context when a decision lands, a trap costs you, a human corrects you, or a human asks. If it is true about this project, not durable in a stronger form, and useful beyond the moment, it is worth a record. Report records you change so a human can review or vouch them.
- **Keep it fresh.** Update affected records with the same change. When code and a record disagree, decide whether implementation drifted from intent or description went stale, then update the stale side; surface a vouched conflict. Back facts with durable evidence such as tests, reproducible commands, committed artifacts, stable URLs, or commit hashes — not ephemeral paths or missing screenshots.
- **Provenance.** Unstamped text is AI-accumulated: challenge and verify it freely. `[VOUCHED @handle YYYY-MM-DD]` means the named human explicitly accepts the covered words as current project direction, not that a factual claim is proven. At a non-heading line's end it covers that line; on its own line as the first nonblank line below a non-title heading it covers that section; on its own line as the first nonblank line below the document title it covers the file. Never put a new stamp in heading text: it breaks link anchors. Legacy stamps before a title or in a heading retain the project's prior scope; never move or reinterpret them without explicit human approval. Add one only on explicit instruction. A stamp added by work under review counts only if the named human confirms it; an unchanged stamp on the target branch is inherited project state. Material edits or scope-boundary changes remove stamps; formatting keeps them only if the covered words stay identical. Legacy undated stamps remain valid until re-vouched.
- **Distill when a human reviews.** Accumulation is noisy by design; the valve is a human review pass. Draft what to prune, merge, or promote, and flag vouches plausibly affected by changes to the areas or evidence they cover. The human decides and vouches.
- **Unattended.** With no human between iterations: keep the running plan as one live record, overwritten as truth changes; tidy your own unstamped layer — merge duplicates, prune dead notes — never the vouched one; when evidence argues with vouched direction, record the conflict and stay inside that direction unless progress becomes impossible; end by drafting the distillation for the returning human, conflicts included. No run, however long or green, vouches anything.
- **The basics.** The recommended starting list — most projects need these; draft the missing ones that apply:
  - `goal.md` — audience, goal, and non-goals; enroll the README instead if it already covers them.
  - `technology-stack.md` — why tools, restrictions, or pins exist; not a manifest dump.
  - `architecture.md` — units, boundaries, and why the lines are where they are.
  - `conventions.md` — deliberate departures from ecosystem defaults.
  - `gotchas.md` — traps already paid for, each with its why.
  - `DESIGN.md` — only for a visual surface; follow https://github.com/google-labs-code/design.md, keep it at the root, and enroll it in the map.
<!-- PCR:END -->
