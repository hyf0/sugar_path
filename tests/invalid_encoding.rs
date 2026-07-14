#![cfg(any(unix, windows))]

use std::{borrow::Cow, path::Path};

use sugar_path::SugarPath;

#[cfg(unix)]
mod unix {
  use std::{
    ffi::OsString,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
  };

  use super::*;

  fn path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(OsString::from_vec(bytes.to_vec()))
  }

  fn assert_bytes(actual: &Path, expected: &[u8]) {
    assert_eq!(actual.as_os_str().as_bytes(), expected);
  }

  fn assert_normalizes_exactly_and_is_idempotent(input: &[u8], expected: &[u8]) {
    let once = path(input).normalize().into_owned();
    assert_bytes(&once, expected);

    let twice = once.normalize().into_owned();
    assert_bytes(&twice, expected);
  }

  #[test]
  fn normalize_preserves_invalid_bytes() {
    assert_normalizes_exactly_and_is_idempotent(b"clean-\x80-name", b"clean-\x80-name");
    assert_normalizes_exactly_and_is_idempotent(
      b"./segment-\x80/./child/../tail/",
      b"segment-\x80/tail/",
    );
  }

  #[cfg(not(target_os = "cygwin"))]
  #[test]
  fn clean_invalid_bytes_borrow_while_dirty_bytes_rebuild() {
    let clean = path(b"/workspace/segment-\x80/file");
    let Cow::Borrowed(normalized) = clean.normalize() else {
      panic!("clean invalid encoding should borrow");
    };
    assert!(std::ptr::eq(normalized, clean.as_path()));

    let dirty = path(b"/workspace/segment-\x80/./file");
    let Cow::Owned(normalized) = dirty.normalize() else {
      panic!("dirty invalid encoding should rebuild");
    };
    assert_bytes(&normalized, b"/workspace/segment-\x80/file");
  }

  #[test]
  fn relative_fallback_preserves_an_invalid_normal_component() {
    let target = path(b"/base/segment-\x80/file");
    let base = path(b"/base");
    let relative = target.relative(&base);

    assert_bytes(&relative, b"segment-\x80/file");
  }

  #[test]
  fn lexical_relative_fast_path_preserves_an_invalid_normal_component() {
    let target = path(b"../../dist/asset-\x80.js");
    let base = path(b"../../dist/chunks");
    let relative = target.relative(&base);

    assert_bytes(&relative, b"../asset-\x80.js");
  }

  #[test]
  fn lexical_relative_common_prefix_compares_invalid_encoding_exactly() {
    let same_base = path(b"../../dist/segment-\x80/chunks");
    let same_target = path(b"../../dist/segment-\x80/assets");
    assert_bytes(&same_target.relative(&same_base), b"../assets");

    assert_eq!(path(b"segment-\x80").to_string_lossy(), path(b"segment-\x81").to_string_lossy());
    let distinct_base = path(b"../../dist/segment-\x80/chunks");
    let distinct_target = path(b"../../dist/segment-\x81/assets");
    assert_bytes(&distinct_target.relative(&distinct_base), b"../../segment-\x81/assets");
  }

  #[test]
  fn non_utf8_slash_policies_preserve_non_normalized_spelling() {
    let input = path(b"./dir//invalid-\x80/../tail/");
    let expected = "./dir//invalid-\u{fffd}/../tail/";

    assert!(input.try_to_slash().is_none());
    assert!(std::panic::catch_unwind(|| input.to_slash()).is_err());
    let lossy = input.to_slash_lossy();
    assert_eq!(lossy, expected);
    assert!(matches!(&lossy, Cow::Owned(_)));
  }

  #[test]
  fn absolutize_with_preserves_invalid_bytes_in_base_and_input() {
    let base = path(b"/workspace/base-\x81");
    let input = path(b"pkg/input-\x80/./file");
    let absolute = input.absolutize_with(base.as_path());

    assert_bytes(&absolute, b"/workspace/base-\x81/pkg/input-\x80/file");
  }
}

#[cfg(windows)]
mod windows {
  use std::{
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
  };

  use super::*;

  const LONE_HIGH_SURROGATE: u16 = 0xd800;

  fn wide_with_invalid(prefix: &str, suffix: &str) -> Vec<u16> {
    wide_with_invalid_unit(prefix, LONE_HIGH_SURROGATE, suffix)
  }

  fn wide_with_invalid_unit(prefix: &str, invalid: u16, suffix: &str) -> Vec<u16> {
    let mut wide: Vec<u16> = prefix.encode_utf16().collect();
    wide.push(invalid);
    wide.extend(suffix.encode_utf16());
    wide
  }

  fn path(wide: &[u16]) -> PathBuf {
    PathBuf::from(OsString::from_wide(wide))
  }

  fn invalid_path(prefix: &str, suffix: &str) -> PathBuf {
    path(&wide_with_invalid(prefix, suffix))
  }

  fn invalid_path_with_unit(prefix: &str, invalid: u16, suffix: &str) -> PathBuf {
    path(&wide_with_invalid_unit(prefix, invalid, suffix))
  }

  fn assert_wide(actual: &Path, expected: &[u16]) {
    assert_eq!(actual.as_os_str().encode_wide().collect::<Vec<_>>(), expected);
  }

  fn assert_relative_cases(cases: &[(&str, &str, &str, &str)]) {
    for &(name, target_prefix, base, expected_prefix) in cases {
      let target = invalid_path(target_prefix, r"\file");
      let relative = target.relative(Path::new(base));
      let actual: Vec<u16> = relative.as_os_str().encode_wide().collect();
      let expected = wide_with_invalid(expected_prefix, r"\file");
      assert_eq!(actual, expected, "{name}");
    }
  }

  fn assert_normalizes_exactly_and_is_idempotent(
    input_prefix: &str,
    input_suffix: &str,
    expected_prefix: &str,
    expected_suffix: &str,
  ) {
    let once = invalid_path(input_prefix, input_suffix).normalize().into_owned();
    let expected = wide_with_invalid(expected_prefix, expected_suffix);
    assert_wide(&once, &expected);

    let twice = once.normalize().into_owned();
    assert_wide(&twice, &expected);
  }

  #[test]
  fn normalize_preserves_invalid_wide_encoding() {
    assert_normalizes_exactly_and_is_idempotent("clean-", "-name", "clean-", "-name");
    assert_normalizes_exactly_and_is_idempotent(
      r".\segment-",
      r"\.\child\..\tail\",
      "segment-",
      r"\tail\",
    );
  }

  #[test]
  fn clean_invalid_wide_encoding_borrows_while_dirty_encoding_rebuilds() {
    let clean = invalid_path(r"C:\workspace\segment-", r"\file");
    let Cow::Borrowed(normalized) = clean.normalize() else {
      panic!("clean invalid encoding should borrow");
    };
    assert!(std::ptr::eq(normalized, clean.as_path()));

    let dirty = invalid_path(r"C:\workspace\segment-", r"\.\file");
    let Cow::Owned(normalized) = dirty.normalize() else {
      panic!("dirty invalid encoding should rebuild");
    };
    assert_wide(&normalized, &wide_with_invalid(r"C:\workspace\segment-", r"\file"));
  }

  #[test]
  fn relative_fallback_preserves_an_invalid_normal_component() {
    assert_relative_cases(&[
      ("disk", r"c:\BASE\segment-", r"C:\base", "segment-"),
      ("verbatim disk", r"\\?\c:\BASE\segment-", r"\\?\C:\base", "segment-"),
      ("UNC", r"\\Server\Share\BASE\segment-", r"\\server\share\base", "segment-"),
      (
        "verbatim UNC",
        r"\\?\UNC\Server\Share\BASE\segment-",
        r"\\?\UNC\server\share\base",
        "segment-",
      ),
      ("device namespace", r"\\.\PIPE\BASE\segment-", r"\\.\pipe\base", "segment-"),
      ("generic verbatim", r"\\?\GLOBALROOT\BASE\segment-", r"\\?\globalroot\base", "segment-"),
    ]);
  }

  #[test]
  fn relative_fallback_keeps_different_windows_namespaces_absolute() {
    assert_relative_cases(&[
      ("different disk", r"c:\base\segment-", r"D:\base", r"c:\base\segment-"),
      ("different verbatim disk", r"\\?\c:\base\segment-", r"\\?\D:\base", r"\\?\c:\base\segment-"),
      (
        "different UNC share",
        r"\\server\share\base\segment-",
        r"\\server\other\base",
        r"\\server\share\base\segment-",
      ),
      (
        "different verbatim UNC server",
        r"\\?\UNC\server\share\base\segment-",
        r"\\?\UNC\other\share\base",
        r"\\?\UNC\server\share\base\segment-",
      ),
      (
        "different device namespace",
        r"\\.\PIPE\base\segment-",
        r"\\.\MAILSLOT\base",
        r"\\.\PIPE\base\segment-",
      ),
      (
        "different generic verbatim namespace",
        r"\\?\GLOBALROOT\base\segment-",
        r"\\?\Volume{abc}\base",
        r"\\?\GLOBALROOT\base\segment-",
      ),
      (
        "ordinary and verbatim disk stay separate",
        r"C:\base\segment-",
        r"\\?\C:\base",
        r"C:\base\segment-",
      ),
      (
        "verbatim and ordinary disk stay separate",
        r"\\?\C:\base\segment-",
        r"C:\base",
        r"\\?\C:\base\segment-",
      ),
      (
        "ordinary and verbatim UNC stay separate",
        r"\\server\share\base\segment-",
        r"\\?\UNC\server\share\base",
        r"\\server\share\base\segment-",
      ),
      (
        "verbatim and ordinary UNC stay separate",
        r"\\?\UNC\server\share\base\segment-",
        r"\\server\share\base",
        r"\\?\UNC\server\share\base\segment-",
      ),
    ]);
  }

  #[test]
  fn lexical_relative_fast_path_preserves_an_invalid_normal_component() {
    let target = invalid_path(r"..\..\dist\asset-", ".js");
    let base = Path::new(r"..\..\dist\chunks");
    let relative = target.relative(base);
    let expected = wide_with_invalid(r"..\asset-", ".js");

    assert_wide(&relative, &expected);
  }

  #[test]
  fn lexical_relative_common_prefix_compares_invalid_encoding_exactly() {
    let same_base = invalid_path(r"..\..\dist\segment-", r"\chunks");
    let same_target = invalid_path(r"..\..\dist\segment-", r"\assets");
    assert_wide(
      &same_target.relative(&same_base),
      &r"..\assets".encode_utf16().collect::<Vec<_>>(),
    );

    const DISTINCT_HIGH_SURROGATE: u16 = 0xd801;
    assert_eq!(
      invalid_path("segment-", "").to_string_lossy(),
      invalid_path_with_unit("segment-", DISTINCT_HIGH_SURROGATE, "").to_string_lossy(),
    );
    let distinct_base = invalid_path(r"..\..\dist\segment-", r"\chunks");
    let distinct_target =
      invalid_path_with_unit(r"..\..\dist\segment-", DISTINCT_HIGH_SURROGATE, r"\assets");
    let expected = wide_with_invalid_unit(r"..\..\segment-", DISTINCT_HIGH_SURROGATE, r"\assets");
    assert_wide(&distinct_target.relative(&distinct_base), &expected);
  }

  #[test]
  fn root_relative_inputs_cancel_the_unknown_drive_without_losing_invalid_encoding() {
    let target = invalid_path(r"\dist\asset-", r"\file");
    let relative = target.relative_with(r"\dist\chunks", "not/absolute");
    let expected = wide_with_invalid(r"..\asset-", r"\file");
    assert_wide(&relative, &expected);
  }

  #[test]
  fn lowercase_unc_marker_stays_in_the_generic_verbatim_namespace() {
    let target = Path::new(r"\\?\unc\server\share\file");
    let base = Path::new(r"\\?\UNC\server\share");
    let relative = target.relative(base);
    let normalized = target.normalize();

    assert!(relative.is_absolute());
    assert_eq!(relative.as_os_str(), normalized.as_os_str());
  }

  #[test]
  fn invalid_unicode_slash_policies_preserve_non_normalized_spelling() {
    let input = invalid_path(r".\dir\\invalid-", r"\..\tail\");
    let expected = "./dir//invalid-\u{fffd}/../tail/";

    assert!(input.try_to_slash().is_none());
    assert!(std::panic::catch_unwind(|| input.to_slash()).is_err());
    let lossy = input.to_slash_lossy();
    assert_eq!(lossy, expected);
    assert!(matches!(&lossy, Cow::Owned(_)));

    let no_native_separator = invalid_path("invalid-", "/tail");
    let lossy = no_native_separator.to_slash_lossy();
    assert_eq!(lossy, "invalid-\u{fffd}/tail");
    assert!(matches!(&lossy, Cow::Owned(_)));
  }

  #[test]
  fn absolutize_with_preserves_invalid_wide_encoding_in_base_and_input() {
    let base = invalid_path(r"C:\workspace\base-", "");
    let input = invalid_path(r"pkg\input-", r"\.\file");
    let absolute = input.absolutize_with(base.as_path());

    let mut expected = wide_with_invalid(r"C:\workspace\base-", r"\pkg\input-");
    expected.push(LONE_HIGH_SURROGATE);
    expected.extend(r"\file".encode_utf16());
    assert_wide(&absolute, &expected);
  }
}
