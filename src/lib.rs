//! Host-native lexical path manipulation with borrowed and consuming APIs.
//!
//! [`SugarPath`] is a sealed extension trait for `Path` and `str`; owned path and string types reach it through normal deref method lookup. [`SugarPathBuf`] contains only consuming operations for which an owned `PathBuf` can be reused. All path operations support the full standard-library native path domain, including non-UTF-8 `OsStr` values.
//!
//! # Quick start
//!
//! ```rust
//! use std::path::Path;
//! use sugar_path::{SugarPath, SugarPathBuf};
//!
//! assert_eq!("foo/./bar/../baz".normalize(), Path::new("foo/baz"));
//!
//! #[cfg(target_family = "unix")]
//! let (target, base, expected) = (Path::new("/workspace/src/index.js"), Path::new("/workspace"), Path::new("src/index.js"));
//! #[cfg(target_family = "windows")]
//! let (target, base, expected) = (Path::new(r"C:\workspace\src\index.js"), Path::new(r"C:\workspace"), Path::new(r"src\index.js"));
//!
//! let relative = target.relative(base);
//! assert_eq!(relative, expected);
//!
//! let slash = relative.into_owned().into_slash();
//! assert_eq!(slash, "src/index.js");
//! ```
//!
//! `normalize` resolves `.` and `..`, collapses redundant native separators, and preserves one trailing separator on a non-root path. It is purely lexical: it does not inspect the filesystem or resolve symlinks. `relative` instead follows Node-style relative output, returning an empty path for equal inputs and removing a target's trailing separator.
//!
//! ```rust
//! use std::path::Path;
//! use sugar_path::SugarPath;
//!
//! #[cfg(target_family = "unix")]
//! assert_eq!(Path::new("foo//bar/").normalize(), Path::new("foo/bar/"));
//!
//! #[cfg(target_family = "windows")]
//! assert_eq!(Path::new("foo\\\\bar\\").normalize(), Path::new("foo\\bar\\"));
//! ```
//!
//! # Current-directory operations
//!
//! [`SugarPath::absolutize`] and [`SugarPath::relative`] read process cwd only when their inputs require it and panic if that lookup fails. [`SugarPath::try_absolutize`] and [`SugarPath::try_relative`] expose the same failure as `io::Error`. [`SugarPath::absolutize_with`] and [`SugarPath::relative_with`] accept an explicit absolute cwd, never read ambient cwd state, and accept either a borrowed path or an owned `PathBuf` directly.
//!
//! ```rust
//! use std::path::{Path, PathBuf};
//! use sugar_path::SugarPath;
//!
//! #[cfg(target_family = "unix")]
//! {
//!   assert_eq!("src/main.rs".absolutize_with(Path::new("/workspace")), Path::new("/workspace/src/main.rs"));
//!   assert_eq!("src/main.rs".absolutize_with(PathBuf::from("/workspace")), Path::new("/workspace/src/main.rs"));
//! }
//!
//! #[cfg(target_family = "windows")]
//! {
//!   assert_eq!(r"src\main.rs".absolutize_with(Path::new(r"C:\workspace")), Path::new(r"C:\workspace\src\main.rs"));
//!   assert_eq!(Path::new("C:foo").absolutize_with(Path::new(r"D:\cwd")), Path::new("C:foo"));
//! }
//! ```
//!
//! An explicit cwd is validated only if the result needs it. On Windows, a drive-relative receiver such as `C:foo` remains drive-relative when an explicit cwd belongs to another drive, because that cwd does not contain drive C's remembered cwd.
//!
//! # Unicode and slash conversion
//!
//! [`SugarPath::to_slash`] and [`SugarPathBuf::into_slash`] are the ergonomic strict conversions and panic for invalid Unicode. The `try_*` forms preserve failure without replacement, while the explicitly `lossy` forms replace invalid encoding with `U+FFFD`.
//!
//! ```rust
//! use std::path::Path;
//! use sugar_path::SugarPath;
//!
//! #[cfg(target_family = "unix")]
//! let path = Path::new("src/main.rs");
//! #[cfg(target_family = "windows")]
//! let path = Path::new(r"src\main.rs");
//!
//! assert_eq!(path.to_slash(), "src/main.rs");
//! assert_eq!(path.try_to_slash().as_deref(), Some("src/main.rs"));
//! ```
//!
//! `relative` returns `Cow<Path>` directly. A canonical descendant can borrow its exact suffix from the receiver; call `Cow::into_owned` only when a `PathBuf` or an owned string is required. The ordinary owned slash-string composition is `target.relative(base).into_owned().into_slash()`.

mod impl_sugar_path;
mod sugar_path;
mod sugar_path_buf;
mod utils;
pub use sugar_path::SugarPath;
pub use sugar_path_buf::SugarPathBuf;
