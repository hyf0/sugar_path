# sugar_path

[![docs.rs](https://docs.rs/sugar_path/badge.svg)](https://docs.rs/sugar_path/latest/sugar_path/)
[![crates.io](https://img.shields.io/crates/v/sugar_path.svg)](https://crates.io/crates/sugar_path)
[![MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Ergonomic path manipulation for Rust — normalize, absolutize, relativize, and slash-convert with zero-cost `Cow` returns.

## Quick start

```bash
cargo add sugar_path
```

```rust
use sugar_path::SugarPath;

// Normalize messy paths
assert_eq!("foo/./bar/../baz".normalize(), "foo/baz".as_path());

// Absolutize relative paths
let abs = "src/main.rs".absolutize();

// Get relative paths between two locations
assert_eq!("/a/b/c/d".as_path().relative("/a/b/f/g"), "../../c/d".as_path());

// Cross-platform slash conversion
assert_eq!("hello/world".as_path().to_slash_lossy(), "hello/world");
```

## API overview

All methods are provided by the `SugarPath` trait, implemented for `Path`, `&str`, and `String`.

### Path conversion

| Method | Description |
|--------|-------------|
| [`as_path()`] | Convert `&str` / `String` to `&Path` — lets you call `SugarPath` methods on strings directly |
| [`to_slash()`] | Convert a path to a `/`-separated string (`None` if invalid UTF-8) |
| [`to_slash_lossy()`] | Like `to_slash()`, but replaces invalid UTF-8 with `U+FFFD` |

```rust
use std::path::Path;
use sugar_path::SugarPath;

assert_eq!("foo".as_path().join("bar"), Path::new("foo/bar"));
```

[`as_path()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.as_path
[`to_slash()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.to_slash
[`to_slash_lossy()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.to_slash_lossy

### Normalization

[`normalize()`] resolves `.` and `..` segments and collapses repeated separators. Returns `Cow::Borrowed` when the path is already clean.

```rust
use std::path::Path;
use sugar_path::SugarPath;

#[cfg(target_family = "unix")]
assert_eq!(
  Path::new("/foo/bar//baz/asdf/quux/..").normalize(),
  Path::new("/foo/bar/baz/asdf")
);
```

[`normalize()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.normalize

### Absolutize

[`absolutize()`] resolves a relative path against the current working directory. [`absolutize_with()`] lets you supply a custom base.

```rust
use std::borrow::Cow;
use sugar_path::SugarPath;

#[cfg(target_family = "unix")]
{
  assert_eq!("./world".absolutize_with(Cow::Borrowed("/hello".as_path())), "/hello/world".as_path());
  assert_eq!("../world".absolutize_with(Cow::Borrowed("/hello".as_path())), "/world".as_path());
}
```

[`absolutize()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.absolutize
[`absolutize_with()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.absolutize_with

### Relative paths

[`relative()`] computes the relative path from `self` to a target.

```rust
use std::path::Path;
use sugar_path::SugarPath;

assert_eq!(Path::new("/base").relative("/base/lib"), Path::new(".."));
assert_eq!(Path::new("/base").relative("/var/lib"), Path::new("../../base"));
assert_eq!(Path::new("/a/b/c/d").relative("/a/b/f/g"), Path::new("../../c/d"));
```

[`relative()`]: https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html#tymethod.relative

## Features

| Feature | Description |
|---------|-------------|
| `cached_current_dir` | Cache `std::env::current_dir()` so `absolutize()` only reads it once |

```toml
sugar_path = { version = "2", features = ["cached_current_dir"] }
```

## Performance

- **Zero-alloc fast paths** — methods return `Cow<'_, Path>`, borrowing the input when no transformation is needed.
- **`memchr`-accelerated scanning** for separator detection.
- **`SmallVec<[_; 8]>`** keeps component lists on the stack for typical path depths.

## Platform support

- Unix and Windows, tested in CI on Ubuntu, macOS, and Windows.
- `to_slash` / `to_slash_lossy` provide consistent `/`-separated output across all platforms.
- On Windows, forward slashes in input are normalized to `\`.

## License

MIT
