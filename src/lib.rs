//! `SugarPath` is a trait that provides a sugar way to manipulate path.
//!
//! [![document](https://docs.rs/sugar_path/badge.svg)](https://docs.rs/crate/sugar_path)
//! [![crate version](https://img.shields.io/crates/v/sugar_path.svg)](https://crates.io/crates/sugar_path)
//! [![MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
//!
//! ## Main functionalities
//! - [SugarPath::as_path] makes it easy to convert `T: Deref<Target = str>` to [Path](std::path::Path) and allows you to use methods of [SugarPath] on `&str` or `String` directly.
//!
//! ```rust
//! use std::path::Path;
//! use sugar_path::SugarPath;
//!
//! assert_eq!("foo".as_path().join("bar"), Path::new("foo/bar"));
//! assert_eq!("foo/./bar/../baz".normalize(), "foo/baz".as_path());
//! ```
//!
//! - [SugarPath::to_slash]/[SugarPath::to_slash_lossy] allows you to convert the path to the string with consistent slash separator on all platforms.
//!
//! ```rust
//! use sugar_path::SugarPath;
//!
//! #[cfg(target_family = "unix")]
//! let p = "./hello/world".as_path();
//!
//! #[cfg(target_family = "windows")]
//! let p = ".\\hello\\world".as_path();
//!
//! assert_eq!(p.to_slash().unwrap(), "./hello/world");
//! assert_eq!(p.to_slash_lossy(), "./hello/world");
//! ```
//! - [SugarPath::normalize] allows you normalize given path by dropping unnecessary `.` or `..` segments.
//!
//! ```rust
//! use std::path::Path;
//! use sugar_path::SugarPath;
//!
//! assert_eq!("foo/./bar/../baz".normalize(), "foo/baz".as_path());
//! ```
//!
//! - [SugarPath::relative] allows you to get the relative path from the given path to the target path.
//!
//! ```rust
//! use sugar_path::SugarPath;
//!
//! assert_eq!("/base".relative("/base/project"), "..".as_path());
//! assert_eq!("/base".relative("/var/lib"), "../../base".as_path());
//! ```
//!
//! - [SugarPath::absolutize] is a shortcut of [SugarPath::absolutize_with] with passing `std::env::current_dir().unwrap()` as the base path.
//! 
//! ```rust
//! use sugar_path::SugarPath;
//! let cwd = std::env::current_dir().unwrap();
//! assert_eq!("hello/world".absolutize(), cwd.join("hello").join("world"));
//! ```
//! 
//! - [SugarPath::absolutize_with] allows you to absolutize the given path with the base path.
//!
//! ```rust
//! use sugar_path::SugarPath;
//! #[cfg(target_family = "unix")]
//! {
//!   assert_eq!("./world".absolutize_with("/hello"), "/hello/world".as_path());
//!   assert_eq!("../world".absolutize_with("/hello"), "/world".as_path());
//! }
//! #[cfg(target_family = "windows")]
//! {
//!  assert_eq!(".\\world".absolutize_with("C:\\hello"), "C:\\hello\\world".as_path());
//!   assert_eq!("..\\world".absolutize_with("C:\\hello"), "C:\\world".as_path());
//! }
//! ```
//!
//! - For more details, please refer to the [SugarPath].

mod impl_sugar_path;
mod sugar_path;
mod utils;
pub use sugar_path::SugarPath;
