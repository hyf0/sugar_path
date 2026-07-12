use std::{
  borrow::Cow,
  io,
  path::{Path, PathBuf},
};

mod private {
  use std::path::Path;

  pub trait Sealed {}

  impl Sealed for Path {}
  impl Sealed for str {}
}

/// Lexical path operations over borrowed standard Rust path and string types.
///
/// Import this trait to call its methods on [`Path`] and `str`. [`PathBuf`],
/// [`String`], and other types that dereference to one of those types use the
/// methods through normal method lookup; they do not implement `SugarPath`
/// themselves.
///
/// The trait is sealed because it is an extension-method namespace, not an
/// abstraction for downstream path types. Generic APIs should accept a
/// standard bound such as [`AsRef<Path>`], then call SugarPath methods on the
/// resulting `&Path`.
///
/// Methods returning [`Cow`] may borrow from the receiver when its existing
/// storage already contains the result. They never borrow from a `base` or
/// `cwd` argument.
///
/// # Generic code
///
/// ```
/// use std::path::{Path, PathBuf};
/// use sugar_path::SugarPath;
///
/// fn normalized(path: impl AsRef<Path>) -> PathBuf {
///   path.as_ref().normalize().into_owned()
/// }
///
/// assert_eq!(normalized(PathBuf::from("src").join("..")), PathBuf::from("."));
/// ```
pub trait SugarPath: private::Sealed {
  /// Lexically normalizes this path in host-native syntax.
  ///
  /// This removes `.` components and redundant separators, resolves `..`
  /// against preceding normal components, and prevents a rooted path from
  /// ascending above its root. An empty path normalizes to `.`. This operation
  /// does not access the filesystem or resolve symlinks; use
  /// [`std::fs::canonicalize`] when physical filesystem identity is required.
  ///
  /// One trailing separator is preserved when the input has one. The returned
  /// [`Cow`] borrows an already-normalized receiver when possible. A canonical
  /// current-directory result may borrow the static `.` path; other results
  /// that require a new buffer are owned.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::path::{Path, PathBuf};
  /// use sugar_path::SugarPath;
  ///
  /// let input = PathBuf::from("workspace").join("src").join("..").join("dist");
  /// let expected = Path::new("workspace").join("dist");
  /// assert_eq!(&*input.normalize(), expected);
  /// ```
  ///
  /// # Windows
  ///
  /// Non-verbatim separators are written as `\`, and the input spelling of a
  /// drive letter is preserved. Verbatim paths retain Rust's native rule that
  /// `/` is a literal character rather than a separator. The minimal `.\` is
  /// kept or inserted when its absence would reinterpret the first normal
  /// component as a prefix.
  fn normalize(&self) -> Cow<'_, Path>;

  /// Resolves this path against the process current directory and normalizes it.
  ///
  /// Resolution removes a non-root trailing separator. An absolute input is
  /// normalized without reading or initializing process cwd state. Other
  /// inputs use the process current directory; with the `cached_current_dir`
  /// feature, ordinary relative inputs use its lazily initialized snapshot.
  ///
  /// A clean absolute receiver may be returned borrowed. A result that requires
  /// cwd resolution is owned.
  ///
  /// # Windows
  ///
  /// On Windows, drive-relative inputs such as `C:foo` use Windows' remembered
  /// current directory for that drive. This lookup is authoritative and is not
  /// replaced by the crate's single cached cwd.
  ///
  /// # Panics
  ///
  /// Panics if required process cwd or Windows drive-cwd state cannot be
  /// resolved. Use [`SugarPath::try_absolutize`] to handle the error.
  fn absolutize(&self) -> Cow<'_, Path>;

  /// Fallible form of [`SugarPath::absolutize`].
  ///
  /// # Errors
  ///
  /// Returns the underlying [`io::Error`] if required ambient cwd state cannot
  /// be obtained or a Windows drive-relative path cannot be made absolute.
  fn try_absolutize(&self) -> io::Result<Cow<'_, Path>>;

  /// Resolves this path against an explicit current directory and normalizes it.
  ///
  /// This method never reads process cwd state. An absolute receiver ignores
  /// `cwd` and may be returned borrowed. A relative result that uses `cwd` is
  /// owned; an owned [`PathBuf`] passed as `cwd` may provide that result buffer.
  /// The returned value never borrows from `cwd`.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::path::Path;
  /// use sugar_path::SugarPath;
  ///
  /// #[cfg(target_family = "unix")]
  /// assert_eq!("src/lib.rs".absolutize_with("/workspace"), Path::new("/workspace/src/lib.rs"));
  ///
  /// #[cfg(target_family = "windows")]
  /// assert_eq!(r"src\lib.rs".absolutize_with(r"C:\workspace"), Path::new(r"C:\workspace\src\lib.rs"));
  /// ```
  ///
  /// # Windows
  ///
  /// On Windows, an ordinary relative path uses `cwd`, and a root-relative path
  /// uses `cwd`'s drive or prefix. A drive-relative receiver such as `C:foo` is
  /// resolved when `cwd` supplies drive C's context. If `cwd` is on another
  /// drive or has a non-disk prefix, the missing drive context is not read from
  /// the environment or invented: the normalized drive-relative receiver is
  /// returned and is not [`Path::is_absolute`].
  ///
  /// # Panics
  ///
  /// Panics if the non-absolute receiver needs `cwd` and `cwd` is not absolute.
  /// An absolute receiver does not inspect or validate `cwd`.
  fn absolutize_with(&self, cwd: impl AsRef<Path> + Into<PathBuf>) -> Cow<'_, Path>;

  /// Returns the lexical path from `base` to this receiver.
  ///
  /// Call this as `target.relative(base)`. Both inputs are resolved as
  /// [`SugarPath::absolutize`] would resolve them, except that cwd-independent
  /// inputs avoid reading ambient cwd state. Equal resolved paths return an
  /// empty path, and result spelling never preserves a non-root target trailing
  /// separator.
  ///
  /// A result already present in the receiver, commonly a descendant suffix
  /// with trailing separators excluded, may be borrowed. Results that must be
  /// rebuilt, including upward and differently rooted results, are owned.
  ///
  /// # Windows
  ///
  /// Drive and path components compare with ASCII case ignored.
  /// Different drive, UNC share, or namespace roots return the normalized
  /// absolute target because a relative path cannot cross those roots. The
  /// normalized target is also returned when its remaining components cannot
  /// be represented by a standalone native relative [`Path`], including a
  /// verbatim component containing literal `/` or a leading component that
  /// would be reparsed as a Windows prefix. This target is normally absolute
  /// after resolution, but can remain root-relative or drive-relative when the
  /// unknown shared context deliberately cancels.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::path::Path;
  /// use sugar_path::SugarPath;
  ///
  /// assert_eq!(Path::new("workspace/src").relative("workspace"), Path::new("src"));
  /// ```
  ///
  /// # Panics
  ///
  /// Panics if required process cwd or Windows drive-cwd state cannot be
  /// resolved. Use [`SugarPath::try_relative`] to handle the error.
  fn relative(&self, base: impl AsRef<Path>) -> Cow<'_, Path>;

  /// Fallible form of [`SugarPath::relative`].
  ///
  /// # Errors
  ///
  /// Returns the underlying [`io::Error`] if either input requires ambient cwd
  /// state that cannot be obtained. Cwd-independent inputs do not produce this
  /// error merely because process cwd is unavailable.
  fn try_relative(&self, base: impl AsRef<Path>) -> io::Result<Cow<'_, Path>>;

  /// Returns the lexical path from `base` to this receiver using `cwd` as the
  /// explicit current directory for relative inputs.
  ///
  /// This method never reads process cwd state. If the result is independent
  /// of cwd, `cwd` is neither inspected nor validated. Otherwise `cwd` resolves
  /// both inputs using [`SugarPath::absolutize_with`]. The returned value may
  /// borrow only from this receiver, never from `base` or `cwd`.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::path::Path;
  /// use sugar_path::SugarPath;
  ///
  /// #[cfg(target_family = "unix")]
  /// assert_eq!("src/lib.rs".relative_with("/workspace", "/workspace"), Path::new("src/lib.rs"));
  ///
  /// #[cfg(target_family = "windows")]
  /// assert_eq!(r"src\lib.rs".relative_with(r"C:\workspace", r"C:\workspace"), Path::new(r"src\lib.rs"));
  /// ```
  ///
  /// # Windows
  ///
  /// On Windows, a single explicit cwd may not contain the remembered cwd for
  /// another drive. Two root-relative inputs cancel their shared unknown drive.
  /// Two drive-relative inputs with the same drive and the same number of
  /// unresolved leading `..` components can likewise cancel that shared
  /// unknown context and produce a relative result. In every other unresolved
  /// or differently rooted case, this method returns the normalized target
  /// instead of fabricating a path relation. That fallback can itself remain
  /// drive-relative and is not guaranteed to satisfy [`Path::is_absolute`].
  /// The unrepresentable-component fallback documented by
  /// [`SugarPath::relative`] also applies.
  ///
  /// # Panics
  ///
  /// Panics if the calculation needs `cwd` and `cwd` is not absolute.
  fn relative_with(
    &self,
    base: impl AsRef<Path>,
    cwd: impl AsRef<Path> + Into<PathBuf>,
  ) -> Cow<'_, Path>;

  /// Converts native separators to `/`, requiring valid UTF-8.
  ///
  /// This operation does not normalize components. It returns a borrowed
  /// string when the path is valid UTF-8 and no separator replacement needs
  /// a new buffer.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::path::PathBuf;
  /// use sugar_path::SugarPath;
  ///
  /// let path = PathBuf::from("src").join("lib.rs");
  /// assert_eq!(path.to_slash(), "src/lib.rs");
  /// ```
  ///
  /// # Panics
  ///
  /// Panics if this native path is not valid UTF-8. Use
  /// [`SugarPath::try_to_slash`] to preserve that failure or
  /// [`SugarPath::to_slash_lossy`] to replace invalid encoding.
  fn to_slash(&self) -> Cow<'_, str>;

  /// Converts native separators to `/`, returning `None` for invalid UTF-8.
  ///
  /// This is the non-panicking strict conversion. It never replaces invalid
  /// native encoding.
  fn try_to_slash(&self) -> Option<Cow<'_, str>>;

  /// Converts native separators to `/`, replacing invalid encoding with the
  /// Unicode replacement character.
  ///
  /// Valid UTF-8 follows the same borrowing behavior as
  /// [`SugarPath::to_slash`]. Replacement is irreversible: a result containing
  /// `U+FFFD` may not round-trip to the original native path. Use
  /// [`SugarPath::try_to_slash`] when the original value must be preserved.
  fn to_slash_lossy(&self) -> Cow<'_, str>;

  /// Views this value as a standard [`Path`] without allocating.
  ///
  /// This is primarily useful for `str` and [`String`] receivers. It performs
  /// no normalization, encoding conversion, or filesystem access.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::path::PathBuf;
  /// use sugar_path::SugarPath;
  ///
  /// assert_eq!("src".as_path().join("lib.rs"), PathBuf::from("src").join("lib.rs"));
  /// ```
  fn as_path(&self) -> &Path;
}
