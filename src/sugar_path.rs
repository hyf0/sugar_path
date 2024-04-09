use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use crate::utils::IntoCowPath;

pub trait SugarPath {
  /// Normalizes the given path, resolving `'..'` and `'.'` segments.
  ///
  /// If normalized path is empty, `'.'` is returned, representing the current working directory.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use std::path::Path;
  /// use sugar_path::SugarPath;
  ///
  /// // For example, on POSIX:
  /// #[cfg(target_family = "unix")]
  /// assert_eq!(
  ///   Path::new("/foo/bar//baz/asdf/quux/..").normalize(),
  ///   Path::new("/foo/bar/baz/asdf")
  /// );
  ///
  /// // On Windows:
  /// #[cfg(target_family = "windows")]
  /// assert_eq!(
  ///   Path::new("C:\\temp\\\\foo\\bar\\..\\").normalize(),
  ///   Path::new("C:\\temp\\foo\\")
  /// );
  ///
  /// // Since Windows recognizes multiple path separators, both separators will be replaced by instances of the Windows preferred separator (`\`):
  /// #[cfg(target_family = "windows")]
  /// assert_eq!(
  ///   Path::new("C:////temp\\\\/\\/\\/foo/bar").normalize(),
  ///   Path::new("C:\\temp\\foo\\bar")
  /// );
  /// ```
  fn normalize(&self) -> PathBuf;

  /// A shortcut of [SugarPath::absolutize_with] with passing `std::env::current_dir().unwrap()` as the base path.
  ///
  /// ## Examples
  /// 
  /// ```rust
  /// use sugar_path::SugarPath;
  /// let cwd = std::env::current_dir().unwrap();
  /// assert_eq!("hello/world".absolutize(), cwd.join("hello").join("world"));
  /// ```
  fn absolutize(&self) -> PathBuf;

  /// Allows you to absolutize the given path with the base path.
  ///
  /// ## Examples
  /// 
  /// ```rust
  /// use sugar_path::SugarPath;
  /// #[cfg(target_family = "unix")]
  /// {
  ///   assert_eq!("./world".absolutize_with("/hello"), "/hello/world".as_path());
  ///   assert_eq!("../world".absolutize_with("/hello"), "/world".as_path());
  /// }
  /// #[cfg(target_family = "windows")]
  /// {
  ///  assert_eq!(".\\world".absolutize_with("C:\\hello"), "C:\\hello\\world".as_path());
  ///   assert_eq!("..\\world".absolutize_with("C:\\hello"), "C:\\world".as_path());
  /// }
  /// ```
  fn absolutize_with<'a>(&self, base: impl IntoCowPath<'a>) -> PathBuf;

  ///
  /// ```rust
  /// use std::path::Path;
  /// use sugar_path::SugarPath;
  /// assert_eq!(
  ///   Path::new("/base").relative("/base/lib"),
  ///   Path::new("..")
  /// );
  /// assert_eq!(
  ///   Path::new("/base").relative("/var/lib"),
  ///   Path::new("../../base")
  /// );
  /// assert_eq!(
  ///   Path::new("/a/b/c/d").relative("/a/b/f/g"),
  ///   Path::new("../../c/d")
  /// );
  /// ```
  fn relative(&self, to: impl AsRef<Path>) -> PathBuf;

  /// [SugarPath::to_slash] converts the path to a string and replaces each separator character with a slash ('/').
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use std::path::Path;
  /// use sugar_path::SugarPath;
  ///
  /// #[cfg(target_family = "unix")]
  /// let p = Path::new("./hello/world");
  ///
  /// #[cfg(target_family = "windows")]
  /// let p = Path::new(".\\hello\\world");
  ///
  /// assert_eq!(p.to_slash().unwrap(), "./hello/world");
  /// ```
  ///
  /// ## In Depth
  ///
  /// When you convert [Path] to [String], you might get different results on different platforms. For `Path::new("./hello/world")`,
  /// you will get `"./hello/world"` on Unix-like systems and `".\\hello\\world"` on Windows. This especially becomes a problem when
  /// your snapshot files of tests contains paths.
  ///
  /// This method solves this problem by converting the path to a string and replacing each separator character with a slash ('/').
  /// So for `Path::new("./hello/world")`, you will get `"./hello/world"` on both Unix-like systems and Windows.
  ///
  /// [SugarPath::to_slash] use [Path::to_str] to convert the path to string under the hood, so it will return `None` if the path contains invalid UTF-8.
  fn to_slash(&self) -> Option<Cow<str>>;

  /// This method is similar to [SugarPath::to_slash], but it use [Path::to_string_lossy] to convert the path to string.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use std::path::Path;
  /// use sugar_path::SugarPath;
  ///
  /// #[cfg(target_family = "unix")]
  /// let p = Path::new("./hello/world");
  ///
  /// #[cfg(target_family = "windows")]
  /// let p = Path::new(".\\hello\\world");
  ///
  /// assert_eq!(p.to_slash_lossy(), "./hello/world");
  /// ```
  fn to_slash_lossy(&self) -> Cow<str>;

  /// An utility method to makes it easy to convert `T: Deref<Target = str>` to [Path](std::path::Path) and allows to you methods of [SugarPath] on `&str` or `String` directly.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use std::path::Path;
  /// use sugar_path::SugarPath;
  ///
  /// assert_eq!("foo".as_path().join("bar"), Path::new("foo/bar"));
  /// assert_eq!("foo/./bar/../baz".normalize(), "foo/baz".as_path());
  /// ```
  fn as_path(&self) -> &Path;
}
