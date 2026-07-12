# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.0.0](https://github.com/hyf0/sugar_path/compare/v2.0.1...v3.0.0) - 2026-07-12

Breaking redesign of the public path APIs for borrowing, explicit cwd, and owned-buffer reuse ([#40](https://github.com/hyf0/sugar_path/pull/40)). Public docs and continuous CI allocation gates were aligned with that surface ([#42](https://github.com/hyf0/sugar_path/pull/42)). Performance baselines landed first as [PR #41](https://github.com/hyf0/sugar_path/pull/41).

### Added

- Add `try_absolutize` and `try_relative` for callers that handle ambient current-directory errors.
- Add `relative_with` for relative calculation against an explicit externally managed cwd.
- Add strict, fallible strict, and explicitly lossy slash conversion for borrowed paths and consuming `PathBuf` values.
- Add consuming `PathBuf` normalization through the sealed `SugarPathBuf` extension trait.

### Changed

- Seal `SugarPath` as an extension-method namespace implemented directly for `Path` and `str`; owned standard types and string wrappers continue to use it through deref method lookup.
- Change `relative` to return `Cow<Path>` directly, borrowing a canonical descendant suffix when possible; callers requiring `PathBuf` use `Cow::into_owned`.
- Follow Node-style host-native normalization by preserving one trailing separator on non-root paths and preserving Windows drive-letter spelling while retaining case-insensitive drive comparison.
- Keep relative output independent from normalization's trailing-separator preservation: equal paths return an empty path and target trailing separators are removed.
- Change `to_slash` to the ergonomic strict conversion that panics for invalid Unicode, and move the non-replacing fallible contract to `try_to_slash`.
- Change `absolutize_with` and `relative_with` to accept borrowed or owned cwd values directly without requiring callers to construct `Cow`; an owned `PathBuf` may transfer its allocation.
- Preserve full non-UTF-8 `Path` and `OsStr` behavior for path operations; only conversion to Unicode chooses strict, fallible, or lossy policy.

### Fixed

- Preserve complete drive, UNC, verbatim UNC, and device roots when calculating Windows relative paths, including invalid-wide fallback inputs.
- Preserve unresolved Windows drive-relative paths when an explicit cwd belongs to another drive instead of consulting ambient state or fabricating a rooted path.
- Preserve literal `/` characters inside Windows verbatim components, keep or insert the minimal `.\` needed to prevent a normal component from becoming a prefix during normalization, and return the normalized target when a component cannot be represented as a standalone native relative path.

### Performance

- Avoid current-directory access for inputs whose result is cwd-independent.
- Avoid result allocation for canonical native descendant `relative` calls on Unix and Windows.
- Reuse owned cwd, normalized `PathBuf`, and valid-Unicode slash-conversion buffers where possible.
- Use separator-aware Windows comparisons, inline component storage, `memchr`, and target-specific common-prefix scanning for Rolldown-shaped paths.

### Migration

- Append `.into_owned()` to `relative(base)` only where the former owned `PathBuf` result is still required.
- For known-UTF-8 owned text, replace `relative(base).to_slash_lossy().into_owned()` with `relative(base).into_owned().into_slash()`; otherwise select the strict fallible or explicitly lossy conversion that matches the caller's encoding policy.
- Replace `to_slash().unwrap()` with `to_slash()` when valid UTF-8 is an invariant, and use `try_to_slash()` when invalid encoding is recoverable.
- Pass cwd directly to `absolutize_with` and `relative_with`; do not wrap it in `Cow::Borrowed` or `Cow::Owned`.

## [2.0.1](https://github.com/hyf0/sugar_path/compare/v2.0.0...v2.0.1) - 2026-02-23

### Fixed

- support wasm platform

### Other

- polish README.md

## [2.0.0](https://github.com/hyf0/sugar_path/compare/v1.2.1...v2.0.0) - 2025-10-21

### Other

- reduce memory allocation ([#26](https://github.com/hyf0/sugar_path/pull/26))
- add deep paths fixtures ([#25](https://github.com/hyf0/sugar_path/pull/25))
- add missing tests and benchmarks for functions used in rolldown ([#24](https://github.com/hyf0/sugar_path/pull/24))
- upgrade all infra ([#21](https://github.com/hyf0/sugar_path/pull/21))
