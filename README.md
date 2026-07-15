# sugar_path

[![CI](https://github.com/hyf0/sugar_path/actions/workflows/test.yaml/badge.svg)](https://github.com/hyf0/sugar_path/actions/workflows/test.yaml)
[![docs.rs](https://docs.rs/sugar_path/badge.svg)](https://docs.rs/sugar_path/latest/sugar_path/)
[![crates.io](https://img.shields.io/crates/v/sugar_path.svg)](https://crates.io/crates/sugar_path)
[![MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/hyf0/sugar_path/blob/main/LICENSE)

Host-native lexical path manipulation for Rust, as extension methods on standard path and string types.

SugarPath adds normalization, absolutization, relative paths, and slash conversion without a wrapper path type. Path-producing methods accept the full native [`Path`](https://doc.rust-lang.org/std/path/struct.Path.html) domain, including non-UTF-8 paths. Conversion to `str` or `String` makes the Unicode policy explicit.

## Installation

```bash
cargo add sugar_path@3
```

CI and local development use the toolchain pinned in [`rust-toolchain.toml`](https://github.com/hyf0/sugar_path/blob/main/rust-toolchain.toml). The crate does not currently declare a separate minimum supported Rust version.

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

| Task | Borrowed / non-consuming | Consuming `PathBuf` | Notes |
| --- | --- | --- | --- |
| Normalize | [`normalize()`] | [`into_normalized()`] | `Cow<Path>` or `PathBuf` |
| Make absolute | [`absolutize()`], [`try_absolutize()`], [`absolutize_with()`] | — | ambient panic · `io::Result` · explicit cwd |
| Make relative | [`relative()`], [`try_relative()`], [`relative_with()`] | — | receiver is the target; returns `Cow<Path>` |
| Convert separators | [`to_slash()`], [`try_to_slash()`], [`to_slash_lossy()`] | [`into_slash()`], [`try_into_slash()`], [`into_slash_lossy()`] | strict · recoverable · lossy Unicode |
| View text as a path | [`as_path()`] | — | borrowed `&Path` |

`PathBuf` and `String` reach [`SugarPath`] methods through normal deref method lookup. Both traits are sealed extension-method namespaces; they are not intended for downstream implementations.

For the full contract of each method — including panic conditions, Windows edge cases, and ownership — see the [crate documentation](https://docs.rs/sugar_path/latest/sugar_path/).

## Semantics

### Lexical paths, not filesystem identity

SugarPath rewrites path components only. It does not touch the filesystem, check existence, or resolve symbolic links. Removing `..` lexically is **not** a security boundary and does not prove filesystem containment. Use [`std::fs::canonicalize`](https://doc.rust-lang.org/std/fs/fn.canonicalize.html) when you need physical filesystem identity.

[`normalize()`] removes `.` components and redundant native separators, resolves `..` against preceding normal components, and preserves one trailing separator on a non-root path. An empty path normalizes to `.`.

### Host-native syntax

Parsing always follows the compilation target's `std::path` rules. There is no caller-selected POSIX/Windows mode, and Windows syntax is not parsed on Unix. On Unix, `\` is an ordinary path byte. On Windows, `/` is normally a separator, but `/` inside a verbatim path component is literal and is left unchanged.

### Relative paths

Call [`relative()`] as `target.relative(base)`. Equal paths return an empty path. A target's non-root trailing separator is removed. A clean descendant may borrow its suffix from the target; upward, rewritten, or differently rooted results are owned.

Ambient cwd is used only when the inputs do not determine the answer themselves. On Windows, paths on different drives, UNC shares, or namespace roots cannot be connected by a native relative path, so the normalized target is returned instead. Drive-relative, root-relative, verbatim, and unrepresentable-component cases are documented on the method.

### Current directory and errors

[`absolutize()`] and [`relative()`] are convenient ambient methods. They panic only when the calculation needs ambient path resolution and that resolution fails. [`try_absolutize()`] and [`try_relative()`] return the underlying `io::Error` instead.

[`absolutize_with()`] and [`relative_with()`] take an explicit absolute cwd and never read process cwd. They accept a borrowed path or an owned `PathBuf`; an owned value may supply reusable result storage. An explicit cwd is validated only when the operation actually needs it.

### Native encoding and slash conversion

Slash conversion changes only the target platform's main separator. It does not normalize components or interpret foreign-platform syntax.

| Policy | Borrowed | Consuming | Invalid native encoding |
| --- | --- | --- | --- |
| Strict | [`to_slash()`] | [`into_slash()`] | panics |
| Recoverable | [`try_to_slash()`] | [`try_into_slash()`] | `None` or original `PathBuf` |
| Lossy | [`to_slash_lossy()`] | [`into_slash_lossy()`] | inserts `U+FFFD`; may not round-trip |

Use strict conversion when valid UTF-8 is an invariant, recoverable conversion when the native path must be kept, and lossy conversion only when replacement is intentional.

## Ownership and allocation

Borrowed `Cow` results never borrow from a `base` or `cwd` argument. They normally borrow from the receiver; normalization may also return the static current-directory path `.`. Already-normalized paths and clean relative descendants can avoid a result allocation. Call `.into_owned()` only when an owned `PathBuf` is required.

[`SugarPathBuf`] consumes `PathBuf` where ownership can avoid a copy. Storage reuse is an optimization: do not rely on the result keeping the same address or capacity. For a final slash-separated `String`, the ordinary composition is:

```rust
use std::path::Path;
use sugar_path::{SugarPath, SugarPathBuf};

let target = Path::new("workspace/src/lib.rs");
let base = Path::new("workspace");
let output = target.relative(base).into_owned().into_slash();
assert_eq!(output, "src/lib.rs");
```

Allocation-sensitive behavior is gated in CI on Linux and Windows via `cargo allocs` (always under `cached_current_dir`). See the [benchmark and allocation methodology](https://github.com/hyf0/sugar_path/blob/main/benchmarks/README.md).

## Cargo features

Default features are empty.

| Feature | Purpose |
| --- | --- |
| `cached_current_dir` | Lazily cache the first successful process cwd lookup for apps that treat cwd as process-lifetime state |
| `codspeed` | Maintainer-only benchmark instrumentation; do not enable in applications |

```toml
sugar_path = { version = "3", features = ["cached_current_dir"] }
```

With `cached_current_dir`, later `std::env::set_current_dir` calls are not observed. Absolute and other cwd-independent operations do not initialize the cache. Failed lookups are not cached. Explicit-cwd methods remain independent. Windows drive-relative paths still use authoritative per-drive cwd resolution.

## Platform support

CI tests Ubuntu, macOS, and Windows with default features and with `cached_current_dir` enabled. It also compile-checks `wasm32-unknown-unknown` and executes selected public contracts under `wasm32-wasip1` with and without cwd caching. Semantics stay host-native:

- Unix paths keep arbitrary native bytes outside Unicode conversion.
- WASIp1 paths keep arbitrary native bytes outside Unicode conversion and use `/` as their separator.
- Windows normalization emits native separators and preserves drive-letter spelling.
- Windows drive, root, and normal-component comparison is ASCII case-insensitive, not general Unicode case folding.
- Windows drive, UNC, verbatim, and device namespaces remain distinct roots.

## Upgrading from 2.x

Version 3 is a breaking API revision:

- [`relative()`] returns `Cow<Path>`; call `.into_owned()` where a `PathBuf` is required.
- [`to_slash()`] is the strict direct-returning conversion; use [`try_to_slash()`] to preserve failure.
- [`SugarPathBuf`] provides consuming normalization and slash conversion.
- Explicit-cwd methods accept borrowed or owned cwd values directly.

See the [changelog migration section](https://github.com/hyf0/sugar_path/blob/main/CHANGELOG.md#migration) for the full checklist.

## License

Licensed under the [MIT License](https://github.com/hyf0/sugar_path/blob/main/LICENSE).

[`SugarPath`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html
[`SugarPathBuf`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPathBuf.html
[`normalize()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.normalize
[`absolutize()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.absolutize
[`try_absolutize()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.try_absolutize
[`absolutize_with()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.absolutize_with
[`relative()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.relative
[`try_relative()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.try_relative
[`relative_with()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.relative_with
[`to_slash()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.to_slash
[`try_to_slash()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.try_to_slash
[`to_slash_lossy()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.to_slash_lossy
[`as_path()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.as_path
[`into_normalized()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPathBuf.html#tymethod.into_normalized
[`into_slash()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPathBuf.html#tymethod.into_slash
[`try_into_slash()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPathBuf.html#tymethod.try_into_slash
[`into_slash_lossy()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPathBuf.html#tymethod.into_slash_lossy
