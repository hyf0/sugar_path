# sugar_path

[![CI](https://github.com/hyf0/sugar_path/actions/workflows/test.yaml/badge.svg)](https://github.com/hyf0/sugar_path/actions/workflows/test.yaml)
[![docs.rs](https://docs.rs/sugar_path/badge.svg)](https://docs.rs/sugar_path/latest/sugar_path/)
[![crates.io](https://img.shields.io/crates/v/sugar_path.svg)](https://crates.io/crates/sugar_path)
[![MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/hyf0/sugar_path/blob/main/LICENSE)

Host-native lexical path manipulation for Rust, exposed as extension methods on standard path and string types.

SugarPath provides normalization, absolutization, relative paths, and slash conversion without introducing a wrapper path type. Path-producing operations support the full native [`Path`](https://doc.rust-lang.org/std/path/struct.Path.html) domain, including non-UTF-8 paths; conversion to `str` or `String` makes the Unicode policy explicit.

> **Release status:** `main` documents the upcoming 3.0 API. The latest published release is 2.0.1; use the [2.0.1 documentation](https://docs.rs/sugar_path/2.0.1/sugar_path/) for the current v2 API. API links in this README point to the v3 source on `main` until its rustdoc is published.

## Installation

After version 3 is published:

```bash
cargo add sugar_path@3
```

To test the unreleased API from `main`:

```toml
[dependencies]
sugar_path = { git = "https://github.com/hyf0/sugar_path.git", branch = "main" }
```

The repository tests with the version pinned in [`rust-toolchain.toml`](https://github.com/hyf0/sugar_path/blob/main/rust-toolchain.toml). The crate does not currently define a separate minimum supported Rust version.

## Quick start

```rust
use std::path::{Path, PathBuf};
use sugar_path::{SugarPath, SugarPathBuf};

let input = PathBuf::from("workspace")
  .join("src")
  .join("..")
  .join("dist")
  .join("assets");

let normalized = input.normalize();
let expected = Path::new("workspace").join("dist").join("assets");
assert_eq!(&*normalized, expected);

// The receiver is the target: target.relative(base).
let relative = normalized.relative("workspace");
assert_eq!(&*relative, Path::new("dist").join("assets"));

let portable = relative.into_owned().into_slash();
assert_eq!(portable, "dist/assets");
```

## Choose an API

Import [`SugarPath`] for borrowed operations on `Path`, `str`, and values that dereference to them. Import [`SugarPathBuf`] for consuming operations that may reuse an owned `PathBuf`.

| Task | Borrowed or non-consuming API | Consuming `PathBuf` API | Result and failure policy |
| --- | --- | --- | --- |
| Normalize | [`normalize()`] | [`into_normalized()`] | `Cow<Path>` or `PathBuf` |
| Make absolute | [`absolutize()`], [`try_absolutize()`], [`absolutize_with()`] | — | ambient panic, `io::Result`, or explicit cwd |
| Make relative | [`relative()`], [`try_relative()`], [`relative_with()`] | — | `Cow<Path>`; the receiver is the target |
| Convert separators | [`to_slash()`], [`try_to_slash()`], [`to_slash_lossy()`] | [`into_slash()`], [`try_into_slash()`], [`into_slash_lossy()`] | strict, recoverable, or lossy Unicode conversion |
| View text as a path | [`as_path()`] | — | borrowed `&Path` |

`PathBuf` and `String` use [`SugarPath`] methods through deref method lookup. The traits are sealed extension-method namespaces; they are not abstractions for downstream implementations.

## Semantics

### Lexical paths, not filesystem canonicalization

SugarPath transforms path components only. It does not access the filesystem, check whether a path exists, or resolve symbolic links. In particular, lexically removing `..` does not prove filesystem containment and must not be used as a security boundary. Use [`std::fs::canonicalize`](https://doc.rust-lang.org/std/fs/fn.canonicalize.html) when physical filesystem identity is required.

[`normalize()`] removes `.` components and redundant native separators, resolves `..` against preceding normal components, and preserves one trailing separator on a non-root path. An empty path normalizes to `.`.

### Host-native syntax

Parsing always follows the compilation target's `std::path` rules. SugarPath does not parse Windows syntax on Unix or provide a caller-selected POSIX/Windows mode. On Unix, `\` is an ordinary path byte. On Windows, `/` is normally accepted as a separator, while `/` inside a verbatim path component is literal and remains unchanged.

### Relative paths

Call [`relative()`] as `target.relative(base)`. Equal paths return an empty path, and a target's non-root trailing separator is removed. A clean descendant may borrow its suffix from the target; upward, rewritten, or differently rooted results are owned.

Relative calculation uses cwd only when the inputs do not determine the answer themselves. On Windows, paths on different drives, UNC shares, or namespace roots cannot be connected by a native relative path, so the normalized target is returned instead. The method rustdoc documents drive-relative, root-relative, verbatim, and unrepresentable-component cases in detail.

### Current directory and errors

[`absolutize()`] and [`relative()`] are convenient ambient methods. They panic only if the calculation requires ambient path resolution and that resolution fails. Their [`try_absolutize()`] and [`try_relative()`] forms return the underlying `io::Error` instead.

[`absolutize_with()`] and [`relative_with()`] use an explicit absolute cwd and never read process cwd state. They accept either a borrowed path or an owned `PathBuf`; an owned value may provide reusable result storage. An explicit cwd is validated only if the operation actually needs it.

### Native encoding and slash conversion

Slash conversion changes only the target platform's main separator. It does not normalize components or interpret foreign-platform syntax.

| Policy | Borrowed API | Consuming API | Invalid native encoding |
| --- | --- | --- | --- |
| Strict | [`to_slash()`] | [`into_slash()`] | panics |
| Recoverable | [`try_to_slash()`] | [`try_into_slash()`] | returns `None` or the original `PathBuf` |
| Lossy | [`to_slash_lossy()`] | [`into_slash_lossy()`] | inserts `U+FFFD`; may not round-trip |

Use strict conversion when valid UTF-8 is an invariant, recoverable conversion when the native path must be preserved, and lossy conversion only when replacement is intentional.

## Ownership and allocation

Borrowed `Cow` results never depend on a `base` or `cwd` lifetime. They normally borrow from the receiver; normalization may also return the static current-directory path `.`. Already-normalized paths and clean relative descendants can therefore avoid a result allocation. Call `.into_owned()` only when an owned `PathBuf` is required.

[`SugarPathBuf`] consumes `PathBuf` for operations where ownership can avoid a copy. Storage reuse is an optimization: callers must not rely on the result retaining the same address or capacity. For a final slash-separated `String`, the ordinary composition is:

```rust
use std::path::Path;
use sugar_path::{SugarPath, SugarPathBuf};

let target = Path::new("workspace/src/lib.rs");
let base = Path::new("workspace");
let output = target.relative(base).into_owned().into_slash();
assert_eq!(output, "src/lib.rs");
```

Allocation-sensitive behavior is checked on macOS, Linux, and Windows. See the [benchmark and allocation methodology](https://github.com/hyf0/sugar_path/blob/main/benchmarks/README.md) for the measured contracts.

## Cargo features

Default features are empty.

| Feature | Purpose |
| --- | --- |
| `cached_current_dir` | Lazily cache the first successful process cwd lookup for applications that treat cwd as process-lifetime state |
| `codspeed` | Maintainer-only benchmark instrumentation; downstream applications should not enable it |

With `cached_current_dir`, later calls do not observe `std::env::set_current_dir`. Absolute and otherwise cwd-independent operations do not initialize the cache, failed lookups are not cached, explicit-cwd methods remain independent, and Windows drive-relative paths still use authoritative per-drive cwd resolution.

## Platform support

CI tests the crate on Ubuntu, macOS, and Windows with default and all features. All semantics remain host-native:

- Unix paths preserve arbitrary native bytes outside Unicode conversion.
- Windows normalization emits native separators and preserves drive-letter spelling.
- Windows drive, root, and normal-component comparison is ASCII case-insensitive, not general Unicode case folding.
- Windows drive, UNC, verbatim, and device namespaces remain distinct roots.

## Upgrading from 2.x

Version 3 is a breaking API revision:

- [`relative()`] now returns `Cow<Path>`; call `.into_owned()` where a `PathBuf` is required.
- [`to_slash()`] is now the strict direct-returning conversion; use [`try_to_slash()`] to preserve failure.
- [`SugarPathBuf`] provides consuming normalization and slash conversion.
- Explicit-cwd methods accept borrowed or owned cwd values directly.

See the [changelog migration section](https://github.com/hyf0/sugar_path/blob/main/CHANGELOG.md#migration) for the complete checklist.

## License

Licensed under the [MIT License](https://github.com/hyf0/sugar_path/blob/main/LICENSE).

[`SugarPath`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`SugarPathBuf`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path_buf.rs
[`normalize()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`absolutize()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`try_absolutize()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`absolutize_with()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`relative()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`try_relative()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`relative_with()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`to_slash()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`try_to_slash()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`to_slash_lossy()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`as_path()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path.rs
[`into_normalized()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path_buf.rs
[`into_slash()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path_buf.rs
[`try_into_slash()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path_buf.rs
[`into_slash_lossy()`]: https://github.com/hyf0/sugar_path/blob/main/src/sugar_path_buf.rs
