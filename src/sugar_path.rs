use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};


pub trait SugarPath {
  /// Normalizes the given path, resolving `'..'` and `'.'` segments.
  ///
  /// If normalized path is empty, `'.'` is returned, representing the current working directory.
  ///
  /// ```rust
  /// use std::path::Path;
  /// use sugar_path::SugarPath;
  ///
  /// // For example, on POSIX:
  /// #[cfg(target_family = "unix")]
  /// assert_eq!(
  ///   Path::new("/foo/bar//baz/asdf/quux/..").normalize().to_slash_lossy(),
  ///   "/foo/bar/baz/asdf"
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

  /// Shortcut for `self.absolutize_with(std::env::current_dir().unwrap())`
  ///
  /// See [SugarPath::absolutize_with] for more details.
  fn absolutize(&self) -> PathBuf;

  /// If the given path is absolute, call [SugarPath::normalize] and return it.
  ///
  /// If the give path is not absolute, it would be joined with the base path, then normalize and return it.
  fn absolutize_with(&self, base: impl Into<PathBuf>) -> PathBuf;

  ///
  /// ```rust
  /// use std::path::Path;
  /// use sugar_path::SugarPath;
  /// assert_eq!(
  ///   Path::new("/var").relative("/var/lib").to_slash_lossy(),
  ///   ".."
  /// );
  /// assert_eq!(
  ///   Path::new("/bin").relative("/var/lib").to_slash_lossy(),
  ///   "../../bin"
  /// );
  /// assert_eq!(
  ///   Path::new("/a/b/c/d").relative("/a/b/f/g").to_slash_lossy(),
  ///   "../../c/d"
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

  /// See [SugarPath::to_slash]
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
  /// 
  /// ## In Depth
  /// 
  /// This method is similar to [SugarPath::to_slash], but it use [Path::to_string_lossy] to convert the path to string.
  fn to_slash_lossy(&self) -> Cow<str>;

  /// An utility method to convert the type to [Path]. This will alow you manipulate the path more easily.
  /// 
  /// ## Examples
  /// 
  /// ```rust
  /// use sugar_path::SugarPath;
  /// 
  /// assert_eq!(
  ///   "hello/.".as_path().join("world").normalize().to_slash_lossy(),
  ///   "hello/world"
  /// );
  /// ```
  fn as_path(&self) -> &Path;
}
