# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Test (all platforms run in CI: ubuntu, macos, windows)
cargo test

# Single test
cargo test --test normalize           # run one test file
cargo test --test normalize unix      # run specific test fn

# Lint (CI runs with -D warnings — zero warnings allowed)
cargo clippy --all-targets --all-features -- -D warnings

# Format (2-space indent, enforced in CI)
cargo fmt --check     # check only
cargo fmt             # fix

# Benchmarks (criterion2, custom harness)
cargo bench -- normalize              # run one bench suite
cargo bench -- absolutize_with        # run matching benchmarks
```

## Architecture

Single-trait library: `SugarPath` trait (`src/sugar_path.rs`) adds path manipulation methods to `Path` and `str`/`String`.

- **`src/sugar_path.rs`** — Trait definition with doc examples
- **`src/impl_sugar_path.rs`** — All implementations. Two impl blocks: one for `Path`, one for `T: Deref<Target = str>`. Contains `normalize_inner()`, `needs_normalization()`, `relative_str()` and helper functions
- **`src/utils.rs`** — `get_current_dir()` helper for `absolutize()`

Key patterns:
- `Cow<'_, Path>` return types to avoid allocation when the input is already clean
- `memchr`-accelerated scanning for separator-based fast paths
- `SmallVec<[_; 8]>` for stack-allocated component lists (8 = typical path depth)
- Platform-specific code via `#[cfg(target_family = "unix")]` / `#[cfg(target_family = "windows")]` — dual implementations of `needs_normalization()` and Windows-specific logic in `relative()`, `normalize_inner()`

## Testing

- **`tests/test_utils.rs`** — Macros: `p!("path")` creates `&Path`, `assert_eq_str!(path_expr, "str")` compares via `.to_str().unwrap()`
- Tests are platform-gated with `#[cfg(target_family = "...")]`
- Benchmarks share fixtures from `benches/fixtures.rs`

## Conventions

- `normalize()` and `normalize_inner()` return `Cow<'_, Path>` — borrowed when input needs no work, owned when normalization occurs
- Performance changes should include benchmark results (`cargo bench`)
- Windows paths use `\` as separator; forward slashes in input trigger normalization
