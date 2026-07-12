# Project goal

## What this library is trying to be

SugarPath is a small Rust library that makes common lexical path operations feel like methods on the standard `Path`, `str`, and `String` types. It should compose with the standard library instead of asking callers to adopt a project-specific path wrapper. Public results stay in standard-library path, string, and `Cow` types.

The intended users are Rust applications and tools that need concise normalization, absolutization, relative-path calculation, or slash conversion while retaining target-native path semantics. The root [README](../../README.md) is the enrolled source for the public feature set and examples.

## What success means

- Common clean inputs avoid allocation where the API returns `Cow`; this is a user-visible performance property, not merely an implementation detail.
- Transformed paths have predictable target-native spelling on Unix and Windows, including exact separators and root forms.
- Deep paths remain correct after the inline `SmallVec` capacity is exceeded; the inline capacity is an optimization, not a supported depth limit.
- The library stays focused enough that borrowed and consuming behavior can be documented through two extension traits over standard path types.

## Non-goals

- SugarPath does not resolve symlinks, inspect path existence, read metadata, or otherwise canonicalize against a filesystem. Normalization is lexical; absolutization adds a base or the process current directory and then normalizes lexically.
- SugarPath does not define an operating-system-independent parser for foreign path syntax. The compilation target's `std::path` rules decide path components; slash conversion only converts the target's main separator for display or snapshots.
- SugarPath does not define a wrapper path type or require callers to convert away from `PathBuf`. `SugarPathBuf` is only a consuming extension trait over the standard type; new consuming methods need evidence that ownership permits real buffer reuse rather than merely duplicating a borrowed method.

## Durable evidence

- [Public trait](../../src/sugar_path.rs)
- [Consuming trait](../../src/sugar_path_buf.rs)
- [Trait implementations](../../src/impl_sugar_path.rs)
- [Allocation assertions](../../tests/normalize.rs)
- [Owned-buffer assertions](../../tests/owned_api.rs)
- [Deep-path coverage](../../tests/deep_paths.rs)
- [Cross-platform CI](../../.github/workflows/test.yaml)
