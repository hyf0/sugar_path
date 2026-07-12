use std::path::PathBuf;

use crate::impl_sugar_path::{
  normalize_owned_path_buf, path_buf_into_slash, path_buf_into_slash_lossy, try_path_buf_into_slash,
};

mod private {
  use std::path::PathBuf;

  pub trait Sealed {}

  impl Sealed for PathBuf {}
}

/// Consuming path operations that can reuse an owned [`PathBuf`] allocation.
///
/// This trait is sealed and implemented only for [`PathBuf`]. Borrowed path
/// operations remain available through [`crate::SugarPath`]. These methods
/// consume the receiver and may reuse its storage; reuse is an optimization,
/// not part of the returned value's semantic contract. Import the trait to use
/// its methods.
pub trait SugarPathBuf: private::Sealed {
  /// Lexically normalizes this path while reusing its allocation when possible.
  ///
  /// The result has the same component, native-separator, drive-spelling, and
  /// trailing-separator semantics as [`crate::SugarPath::normalize`].
  ///
  /// # Examples
  ///
  /// ```
  /// use std::path::{Path, PathBuf};
  /// use sugar_path::SugarPathBuf;
  ///
  /// let input = PathBuf::from("workspace").join("src").join("..").join("dist");
  /// let expected = Path::new("workspace").join("dist");
  /// assert_eq!(input.into_normalized(), expected);
  /// ```
  #[must_use]
  fn into_normalized(self) -> PathBuf;

  /// Converts native separators to `/`, requiring valid UTF-8.
  ///
  /// This has the same strict, non-normalizing conversion semantics as
  /// [`crate::SugarPath::to_slash`] and may reuse the consumed path's storage.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::path::PathBuf;
  /// use sugar_path::SugarPathBuf;
  ///
  /// assert_eq!(PathBuf::from("src").join("lib.rs").into_slash(), "src/lib.rs");
  /// ```
  ///
  /// # Panics
  ///
  /// Panics if this native path is not valid UTF-8. Use
  /// [`SugarPathBuf::try_into_slash`] to recover the original input or
  /// [`SugarPathBuf::into_slash_lossy`] to replace invalid encoding.
  #[must_use]
  fn into_slash(self) -> String;

  /// Converts native separators to `/` without replacing invalid encoding.
  ///
  /// Returns `Ok(String)` for valid UTF-8.
  ///
  /// # Errors
  ///
  /// Returns `Err` when the native path is not valid UTF-8. The error contains
  /// the original [`PathBuf`] unchanged, including its native encoding and
  /// spelling, so the caller can choose another representation.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::{ffi::OsString, path::PathBuf};
  /// use sugar_path::SugarPathBuf;
  ///
  /// #[cfg(target_family = "unix")]
  /// {
  ///   use std::os::unix::ffi::OsStringExt;
  ///   let path = PathBuf::from(OsString::from_vec(vec![0xff]));
  ///   assert_eq!(path.clone().try_into_slash(), Err(path));
  /// }
  ///
  /// #[cfg(target_family = "windows")]
  /// {
  ///   use std::os::windows::ffi::OsStringExt;
  ///   let path = PathBuf::from(OsString::from_wide(&[0xd800]));
  ///   assert_eq!(path.clone().try_into_slash(), Err(path));
  /// }
  /// ```
  fn try_into_slash(self) -> Result<String, PathBuf>;

  /// Converts native separators to `/`, replacing invalid encoding with the
  /// Unicode replacement character.
  ///
  /// This conversion always returns a [`String`] and may reuse storage when the
  /// input is valid UTF-8. Replacement is irreversible: a result containing
  /// `U+FFFD` may not round-trip to the original native path. Use
  /// [`SugarPathBuf::try_into_slash`] when the original value must be preserved.
  #[must_use]
  fn into_slash_lossy(self) -> String;
}

impl SugarPathBuf for PathBuf {
  fn into_normalized(self) -> PathBuf {
    normalize_owned_path_buf(self)
  }

  fn into_slash(self) -> String {
    path_buf_into_slash(self)
  }

  fn try_into_slash(self) -> Result<String, PathBuf> {
    try_path_buf_into_slash(self)
  }

  fn into_slash_lossy(self) -> String {
    path_buf_into_slash_lossy(self)
  }
}
