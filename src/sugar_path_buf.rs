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
/// not part of the returned value's semantic contract.
pub trait SugarPathBuf: private::Sealed {
  /// Lexically normalizes this path while reusing its allocation when possible.
  ///
  /// The result has the same component, native-separator, drive-spelling, and
  /// trailing-separator semantics as [`crate::SugarPath::normalize`].
  #[must_use]
  fn into_normalized(self) -> PathBuf;

  /// Converts native separators to `/`, requiring valid UTF-8.
  ///
  /// This has the same strict, non-normalizing conversion semantics as
  /// [`crate::SugarPath::to_slash`] and may reuse the consumed path's storage.
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
  /// Returns `Ok(String)` for valid UTF-8. On failure, `Err` contains the
  /// original [`PathBuf`] unchanged, including its native encoding and
  /// spelling.
  fn try_into_slash(self) -> Result<String, PathBuf>;

  /// Converts native separators to `/`, replacing invalid encoding with the
  /// Unicode replacement character.
  ///
  /// This conversion always returns a [`String`] and may reuse storage when the
  /// input is valid UTF-8.
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
