#![deny(unsafe_code)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![warn(missing_docs, rustdoc::broken_intra_doc_links)]
//! Host-native lexical path manipulation as extension methods on standard Rust types.
//!
//! SugarPath adds normalization, absolutization, relative paths, and slash
//! conversion without introducing a wrapper path type. Path-producing methods
//! accept the full native [`Path`](std::path::Path) domain, including non-UTF-8 paths; conversion
//! to [`str`] or [`String`] makes the Unicode policy explicit.
//!
//! Import [`SugarPath`] for borrowed operations on [`Path`](std::path::Path) and `str`. Values
//! such as [`PathBuf`](std::path::PathBuf) and [`String`] use the same methods through deref method
//! lookup. Import [`SugarPathBuf`] for consuming operations that may reuse an
//! owned path buffer.
//!
//! # Quick start
//!
//! ```rust
//! use std::path::{Path, PathBuf};
//! use sugar_path::{SugarPath, SugarPathBuf};
//!
//! let input = PathBuf::from("workspace")
//!   .join("src")
//!   .join("..")
//!   .join("dist")
//!   .join("assets");
//!
//! let normalized = input.normalize();
//! let expected = Path::new("workspace").join("dist").join("assets");
//! assert_eq!(&*normalized, expected);
//!
//! // The receiver is the target: target.relative(base).
//! let relative = normalized.relative("workspace");
//! assert_eq!(&*relative, Path::new("dist").join("assets"));
//! assert_eq!(relative.into_owned().into_slash(), "dist/assets");
//! ```
//!
//! # Choosing an API
//!
//! | Task | Borrowed or non-consuming | Consuming [`PathBuf`](std::path::PathBuf) |
//! | --- | --- | --- |
//! | Normalize | [`SugarPath::normalize`] | [`SugarPathBuf::into_normalized`] |
//! | Make absolute | [`SugarPath::absolutize`], [`SugarPath::try_absolutize`], [`SugarPath::absolutize_with`] | — |
//! | Make relative | [`SugarPath::relative`], [`SugarPath::try_relative`], [`SugarPath::relative_with`] | — |
//! | Convert separators | [`SugarPath::to_slash`], [`SugarPath::try_to_slash`], [`SugarPath::to_slash_lossy`] | [`SugarPathBuf::into_slash`], [`SugarPathBuf::try_into_slash`], [`SugarPathBuf::into_slash_lossy`] |
//! | View text as a path | [`SugarPath::as_path`] | — |
//!
//! The ambient [`SugarPath::absolutize`] and [`SugarPath::relative`] methods
//! panic only when required ambient path resolution fails. Their `try_*` forms
//! expose the same failure as [`std::io::Error`]. Prefer the `*_with` methods
//! when the base directory is known: they take an explicit cwd and never read
//! ambient cwd state.
//!
//! Strict slash conversion panics for invalid Unicode, fallible conversion
//! preserves failure without replacement, and only methods named `lossy`
//! insert `U+FFFD`. Each method page documents panic conditions, Windows edge
//! cases, and when a [`Cow`](std::borrow::Cow) result may borrow.
//!
//! # Lexical and host-native semantics
//!
//! These operations transform path components only. They do not access the
//! filesystem, check whether a path exists, or resolve symbolic links.
//! Lexically removing `..` therefore does not prove filesystem containment and
//! must not be used as a security boundary. Use [`std::fs::canonicalize`] when
//! physical filesystem identity is required.
//!
//! Parsing follows the compilation target's [`std::path`] rules. SugarPath
//! does not parse Windows syntax on Unix or expose a caller-selected path
//! syntax. [`SugarPath::normalize`] preserves one trailing separator on a
//! non-root path. [`SugarPath::relative`] returns an empty path for equal inputs
//! and removes a target's non-root trailing separator.
//!
//! # Ownership and native encoding
//!
//! Borrowed [`Cow`](std::borrow::Cow) results never depend on a `base` or `cwd`
//! lifetime. They normally borrow from the receiver; normalization may also
//! return the static current-directory path `.`. An already-normalized path or
//! clean relative descendant can therefore avoid a result allocation. Results
//! that require a new buffer are owned. Call
//! [`Cow::into_owned`](std::borrow::Cow::into_owned) only when an owned
//! [`PathBuf`](std::path::PathBuf) is required.
//!
//! Path-producing operations preserve arbitrary native encoding. Slash
//! conversion is the boundary where callers choose strict, recoverable, or
//! lossy Unicode behavior.
//!
//! # Cargo features
//!
//! - `cached_current_dir` caches the first successful ambient cwd lookup for
//!   processes that treat cwd as stable. Later `std::env::set_current_dir`
//!   calls are not observed. Explicit-cwd methods remain independent, and
//!   Windows drive-relative paths still use authoritative per-drive cwd state.
//! - `codspeed` enables maintainer benchmark instrumentation and is not intended
//!   for downstream applications.
//!
//! See the [README](https://github.com/hyf0/sugar_path) for platform notes and
//! the [changelog](https://github.com/hyf0/sugar_path/blob/main/CHANGELOG.md)
//! for release and migration information.

#[cfg(not(unix))]
mod encoded_arena;
mod impl_sugar_path;
mod sugar_path;
mod sugar_path_buf;
mod utils;
pub use sugar_path::SugarPath;
pub use sugar_path_buf::SugarPathBuf;
