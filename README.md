# sugar_path

[![docs.rs](https://docs.rs/sugar_path/badge.svg)](https://docs.rs/sugar_path/latest/sugar_path/)
[![crates.io](https://img.shields.io/crates/v/sugar_path.svg)](https://crates.io/crates/sugar_path)
[![MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Ergonomic host-native path manipulation for Rust, with lexical normalization, absolutization, relative paths, slash conversion, borrowing, and owned-buffer reuse.

SugarPath operates on the full standard-library `Path` domain, including non-UTF-8 native paths. It follows the compilation target's `std::path` parsing and separator rules; it does not parse Windows paths on Unix or POSIX paths on Windows.

## Quick start

```bash
cargo add sugar_path
```

```rust
use std::path::Path;
use sugar_path::{SugarPath, SugarPathBuf};

assert_eq!("foo/./bar/../baz".normalize(), Path::new("foo/baz"));

#[cfg(target_family = "unix")]
let (target, base, expected) = ("/workspace/src/index.js", "/workspace", "src/index.js");
#[cfg(target_family = "windows")]
let (target, base, expected) = (r"C:\workspace\src\index.js", r"C:\workspace", r"src\index.js");

let relative = Path::new(target).relative(base);
assert_eq!(relative, Path::new(expected));

let slash = relative.into_owned().into_slash();
assert_eq!(slash, "src/index.js");
```

## API overview

`SugarPath` is a sealed extension trait implemented directly for `Path` and `str`. `PathBuf`, `String`, and string wrappers use the same methods through normal deref method lookup. `SugarPathBuf` is a separate sealed trait for consuming operations that can reuse an owned `PathBuf` allocation.

### Slash conversion

| Method | Behavior |
| --- | --- |
| [`as_path()`] | View a string as `&Path` without allocating |
| [`to_slash()`] | Return a borrowed or owned slash-separated UTF-8 string; panic for invalid Unicode |
| [`try_to_slash()`] | Return `None` for invalid Unicode without replacement |
| [`to_slash_lossy()`] | Replace invalid encoding with `U+FFFD` |
| [`into_slash()`] | Consume a valid-Unicode `PathBuf` and reuse its storage when possible; panic for invalid Unicode |
| [`try_into_slash()`] | Return the original `PathBuf` when strict consuming conversion fails |
| [`into_slash_lossy()`] | Consume a `PathBuf` and replace invalid encoding |

```rust
use std::path::Path;
use sugar_path::SugarPath;

#[cfg(target_family = "unix")]
let path = Path::new("src/main.rs");
#[cfg(target_family = "windows")]
let path = Path::new(r"src\main.rs");

assert_eq!(path.to_slash(), "src/main.rs");
assert_eq!(path.try_to_slash().as_deref(), Some("src/main.rs"));
```

Use the strict methods when valid UTF-8 is part of the caller's contract. Use the `try_*` methods when invalid native encoding must be preserved, and use the explicitly lossy methods only when replacement is intended.

[`as_path()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.as_path
[`to_slash()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.to_slash
[`try_to_slash()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.try_to_slash
[`to_slash_lossy()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.to_slash_lossy
[`into_slash()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPathBuf.html#tymethod.into_slash
[`try_into_slash()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPathBuf.html#tymethod.try_into_slash
[`into_slash_lossy()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPathBuf.html#tymethod.into_slash_lossy

### Normalization

[`normalize()`] resolves `.` and `..`, collapses redundant native separators, and preserves one trailing separator on a non-root path. It is lexical and does not inspect the filesystem or resolve symlinks. A clean result may borrow the receiver.

[`into_normalized()`] performs the same operation on an owned `PathBuf` and reuses that allocation when possible.

```rust
use std::path::Path;
use sugar_path::SugarPath;

#[cfg(target_family = "unix")]
assert_eq!(Path::new("foo//bar/").normalize(), Path::new("foo/bar/"));

#[cfg(target_family = "windows")]
assert_eq!(Path::new(r"foo\\bar\").normalize(), Path::new(r"foo\bar\"));
```

Unlike the former canonical-spelling contract, Windows normalization preserves the input spelling of a drive letter. Drive and root comparison remains ASCII case-insensitive. Under a Windows verbatim prefix, `/` is a literal character rather than a separator and is preserved exactly, following `std::path`. Normalization also keeps or inserts the minimal `.\` when its absence would cause a normal component such as `C:foo` to be reparsed as a drive prefix.

[`normalize()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.normalize
[`into_normalized()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPathBuf.html#tymethod.into_normalized

### Absolutization and cwd

[`absolutize()`] normalizes an absolute receiver directly or resolves a relative receiver against the process current directory. [`try_absolutize()`] exposes a current-directory lookup failure instead of panicking. Absolute inputs do not read or initialize cwd state.

[`absolutize_with()`] accepts an explicit absolute cwd and never reads process cwd state. Pass `&Path` to borrow an existing cwd, or pass an owned `PathBuf` to transfer storage that may be reused; callers do not construct `Cow` explicitly.

```rust
use std::path::{Path, PathBuf};
use sugar_path::SugarPath;

#[cfg(target_family = "unix")]
{
  assert_eq!("src/main.rs".absolutize_with(Path::new("/workspace")), Path::new("/workspace/src/main.rs"));
  assert_eq!("src/main.rs".absolutize_with(PathBuf::from("/workspace")), Path::new("/workspace/src/main.rs"));
}

#[cfg(target_family = "windows")]
{
  assert_eq!(r"src\main.rs".absolutize_with(Path::new(r"C:\workspace")), Path::new(r"C:\workspace\src\main.rs"));
  assert_eq!(r"src\main.rs".absolutize_with(PathBuf::from(r"C:\workspace")), Path::new(r"C:\workspace\src\main.rs"));
}
```

An explicit cwd is validated only when an operation needs it. A relative receiver with a non-absolute explicit cwd violates the contract and panics; an absolute receiver ignores an unused cwd.

[`absolutize()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.absolutize
[`try_absolutize()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.try_absolutize
[`absolutize_with()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.absolutize_with

### Relative paths

[`relative()`] returns the lexical path from its argument to the receiver: call `target.relative(base)`. It returns `Cow<Path>` directly. A clean descendant may borrow its exact suffix from the target; rebuilt, upward, dirty, or differently rooted results are owned. Call `Cow::into_owned` only when a `PathBuf` is required.

Equal paths return an empty path. Relative output follows Node-style resolution and removes a target's trailing separator even though `normalize()` preserves one.

On Windows, different drives, UNC shares, or namespace roots return the normalized absolute target. SugarPath returns the normalized target when components cannot be represented as a standalone native relative `Path`, such as a verbatim component containing literal `/` or a leading component that would be parsed as a drive prefix. That result is normally absolute, but can remain root-relative or drive-relative when the inputs intentionally cancel a shared unknown context.

[`try_relative()`] exposes ambient cwd errors. [`relative_with()`] accepts an explicit absolute cwd for relative inputs and never reads process cwd state.

```rust
use std::{borrow::Cow, path::Path};
use sugar_path::SugarPath;

#[cfg(target_family = "unix")]
let (target, base, expected) = (Path::new("/workspace/src/index.js"), Path::new("/workspace"), Path::new("src/index.js"));
#[cfg(target_family = "windows")]
let (target, base, expected) = (Path::new(r"C:\workspace\src\index.js"), Path::new(r"C:\workspace"), Path::new(r"src\index.js"));

let relative = target.relative(base);
assert_eq!(relative, expected);
assert!(matches!(relative, Cow::Borrowed(_)));
assert_eq!(target.relative(target), Path::new(""));
```

[`relative()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.relative
[`try_relative()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.try_relative
[`relative_with()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.relative_with

## Features

| Feature | Description |
| --- | --- |
| `cached_current_dir` | Cache process cwd when an ambient operation first needs it, for applications that treat cwd as stable |

```toml
sugar_path = { version = "3", features = ["cached_current_dir"] }
```

Explicit-cwd methods remain independent from the cache and should be used when cwd is externally managed or may change.

## Performance

- `Cow` results borrow clean normalized paths and canonical descendant relative suffixes.
- Consuming `PathBuf` methods reuse owned storage when possible.
- Separator-aware scans avoid allocating normalized copies merely to compare clean Windows paths.
- `memchr`, inline component storage, and target-specific common-prefix scanning cover common Rolldown path shapes.

## Platform behavior

- Unix and Windows are tested in CI on Ubuntu, macOS, and Windows.
- Path parsing and normalization always use host-native `std::path` semantics.
- Windows drive and root comparison is ASCII case-insensitive, while normalization preserves drive-letter spelling.
- Windows verbatim paths keep `std::path` separator rules: `/` is a literal character and is not rewritten as `\`.
- Different Windows drives or UNC shares cannot be crossed by a relative path, so `relative` returns the normalized target.
- A Windows target remains in normalized target form when its components cannot be represented by a standalone native relative `Path` without changing their meaning.
- For an explicit cwd on another drive, `Path::new("C:foo").absolutize_with("D:\\cwd")` preserves the normalized drive-relative `C:foo`; it does not consult ambient state or fabricate `C:\\foo`.
- `relative_with` relates two root-relative inputs without reading a drive and relates same-drive drive-relative inputs when their unknown shared context provably cancels; otherwise it returns the normalized target.

## License

MIT
