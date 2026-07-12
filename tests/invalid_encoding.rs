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
  use sugar_path::SugarPathBuf;

  fn path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(OsString::from_vec(bytes.to_vec()))
  }

  fn assert_bytes(actual: &Path, expected: &[u8]) {
    assert_eq!(actual.as_os_str().as_bytes(), expected);
  }

  fn assert_relative_variants(target: &Path, base: &Path, expected: &[u8]) {
    assert_bytes(&target.relative(base), expected);
    assert_bytes(&target.try_relative(base).expect("absolute paths do not need cwd"), expected);
    assert_bytes(&target.relative_with(base, Path::new("/")), expected);
  }

  fn assert_normalizes_exactly_and_is_idempotent(input: &[u8], expected: &[u8]) {
    let source = path(input);
    let borrowed = source.normalize().into_owned();
    let identity = (source.as_os_str().as_bytes().as_ptr(), source.capacity());
    let owned = source.into_normalized();
    assert_eq!(
      (owned.as_os_str().as_bytes().as_ptr(), owned.capacity()),
      identity,
      "owned normalization did not reuse its input buffer",
    );

    for once in [borrowed, owned] {
      assert_bytes(&once, expected);
      let twice = once.normalize().into_owned();
      assert_bytes(&twice, expected);
    }
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
  fn normalize_preserves_arbitrary_bytes_at_component_and_arena_boundaries() {
    assert_normalizes_exactly_and_is_idempotent(
      b"\xff-first-\x80/./\xc0-second-\0/drop-\xed\xa0\x80/../tail/",
      b"\xff-first-\x80/\xc0-second-\0/tail/",
    );

    let mut input = b"./".to_vec();
    input.resize(511, b'x');
    input.push(0xff);
    assert_eq!(input.len(), 512);

    let mut expected = vec![b'x'; 509];
    expected.push(0xff);
    assert_normalizes_exactly_and_is_idempotent(&input, &expected);
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
  fn absolute_relative_common_prefix_compares_invalid_encoding_exactly() {
    let same_base = path(b"/workspace/segment-\x80/chunks");
    let same_target = path(b"/workspace/segment-\x80/assets");
    assert_relative_variants(&same_target, &same_base, b"../assets");

    assert_eq!(path(b"segment-\x80").to_string_lossy(), path(b"segment-\x81").to_string_lossy());
    let distinct_base = path(b"/workspace/segment-\x80/chunks");
    let distinct_target = path(b"/workspace/segment-\x81/assets");
    assert_relative_variants(&distinct_target, &distinct_base, b"../../segment-\x81/assets");
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

  #[test]
  fn relative_with_preserves_invalid_bytes_in_cwd() {
    let cwd = path(b"/anchor/invalid-\x80/leaf");
    let target = Path::new("../target");
    let base = Path::new("../../base");

    let borrowed_cwd = target.relative_with(base, cwd.as_path());
    assert_bytes(&borrowed_cwd, b"../invalid-\x80/target");
    assert!(matches!(borrowed_cwd, Cow::Owned(_)));

    let owned_cwd = target.relative_with(base, cwd);
    assert_bytes(&owned_cwd, b"../invalid-\x80/target");
    assert!(matches!(owned_cwd, Cow::Owned(_)));
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
  use sugar_path::SugarPathBuf;

  const LONE_HIGH_SURROGATE: u16 = 0xd800;
  const DISTINCT_HIGH_SURROGATE: u16 = 0xd801;

  #[derive(Clone, Copy)]
  enum WidePart<'a> {
    Text(&'a str),
    Unit(u16),
  }

  #[derive(Clone, Copy, Debug)]
  enum PrefixClass {
    None,
    Disk,
    VerbatimDisk,
    Unc,
    VerbatimUnc,
    DeviceNs,
    Verbatim,
  }

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

  fn wide(parts: &[WidePart<'_>]) -> Vec<u16> {
    let mut result = Vec::new();
    for part in parts {
      match *part {
        WidePart::Text(text) => result.extend(text.encode_utf16()),
        WidePart::Unit(unit) => result.push(unit),
      }
    }
    result
  }

  fn assert_prefix_class(name: &str, path: &Path, expected: PrefixClass) {
    use std::path::{Component, Prefix};

    let actual = match path.components().next() {
      Some(Component::Prefix(prefix)) => Some(prefix.kind()),
      _ => None,
    };
    let matches = matches!(
      (expected, actual),
      (PrefixClass::None, None)
        | (PrefixClass::Disk, Some(Prefix::Disk(_)))
        | (PrefixClass::VerbatimDisk, Some(Prefix::VerbatimDisk(_)))
        | (PrefixClass::Unc, Some(Prefix::UNC(_, _)))
        | (PrefixClass::VerbatimUnc, Some(Prefix::VerbatimUNC(_, _)))
        | (PrefixClass::DeviceNs, Some(Prefix::DeviceNS(_)))
        | (PrefixClass::Verbatim, Some(Prefix::Verbatim(_)))
    );
    assert!(matches, "{name}: fixture parsed as {actual:?}");
  }

  fn assert_wide_case(name: &str, stage: &str, actual: &Path, expected: &[u16]) {
    assert_eq!(actual.as_os_str().encode_wide().collect::<Vec<_>>(), expected, "{name}: {stage}",);
  }

  fn assert_normalization_case(
    name: &str,
    prefix: PrefixClass,
    input: &[WidePart<'_>],
    expected: &[WidePart<'_>],
  ) {
    let source = path(&wide(input));
    let expected = wide(expected);

    assert_prefix_class(name, &source, prefix);
    assert!(source.to_str().is_none(), "{name}: fixture must contain unpaired surrogates");
    assert!(source.as_os_str().len() <= 512, "{name}: fixture must use the arena path");

    let borrowed = source.normalize().into_owned();
    let owned_input = source.clone();
    let identity = (owned_input.as_os_str().as_encoded_bytes().as_ptr(), owned_input.capacity());
    let owned = owned_input.into_normalized();
    assert_eq!(
      (owned.as_os_str().as_encoded_bytes().as_ptr(), owned.capacity()),
      identity,
      "{name}: owned normalization did not reuse its input buffer",
    );

    for (api, once) in [("normalize", borrowed), ("into_normalized", owned)] {
      assert_wide_case(name, api, &once, &expected);
      assert!(once.to_str().is_none(), "{name}: {api} paired or lost a surrogate");

      let twice_borrowed = once.normalize();
      assert_wide_case(name, "idempotent normalize", twice_borrowed.as_ref(), &expected);
      drop(twice_borrowed);

      let twice_owned = once.into_normalized();
      assert_wide_case(name, "idempotent into_normalized", &twice_owned, &expected);
    }
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

  fn assert_relative_variants(target: &Path, base: &Path, expected: &[u16]) {
    assert_wide(&target.relative(base), expected);
    assert_wide(&target.try_relative(base).expect("absolute paths do not need cwd"), expected);
    assert_wide(&target.relative_with(base, Path::new(r"C:\unused")), expected);
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
  fn normalization_preserves_surrogates_at_every_windows_chunk_boundary() {
    use WidePart::{Text as T, Unit as U};

    const HIGH: u16 = 0xd800;
    const LOW: u16 = 0xdc00;

    assert_normalization_case(
      "relative normal components",
      PrefixClass::None,
      &[U(LOW), T("first"), U(HIGH), T(r"\.\"), U(LOW), T("second"), U(HIGH), T(r"\drop\..\tail")],
      &[U(LOW), T("first"), U(HIGH), T(r"\"), U(LOW), T("second"), U(HIGH), T(r"\tail")],
    );

    assert_normalization_case(
      "drive-relative disk",
      PrefixClass::Disk,
      &[T("c:"), U(LOW), T("disk"), U(HIGH), T(r"\.\tail")],
      &[T("c:"), U(LOW), T("disk"), U(HIGH), T(r"\tail")],
    );

    assert_normalization_case(
      "verbatim disk",
      PrefixClass::VerbatimDisk,
      &[T(r"\\?\c:\"), U(LOW), T("disk"), U(HIGH), T(r"\.\tail")],
      &[T(r"\\?\c:\"), U(LOW), T("disk"), U(HIGH), T(r"\tail")],
    );

    assert_normalization_case(
      "UNC fields",
      PrefixClass::Unc,
      &[
        T(r"\\"),
        U(LOW),
        T("server"),
        U(HIGH),
        T(r"\"),
        U(LOW),
        T("share"),
        U(HIGH),
        T(r"\"),
        U(LOW),
        T("leaf"),
        U(HIGH),
        T(r"\.\tail"),
      ],
      &[
        T(r"\\"),
        U(LOW),
        T("server"),
        U(HIGH),
        T(r"\"),
        U(LOW),
        T("share"),
        U(HIGH),
        T(r"\"),
        U(LOW),
        T("leaf"),
        U(HIGH),
        T(r"\tail"),
      ],
    );

    assert_normalization_case(
      "verbatim UNC fields",
      PrefixClass::VerbatimUnc,
      &[
        T(r"\\?\UNC\"),
        U(LOW),
        T("server"),
        U(HIGH),
        T(r"\"),
        U(LOW),
        T("share"),
        U(HIGH),
        T(r"\"),
        U(LOW),
        T("leaf"),
        U(HIGH),
        T(r"\.\tail"),
      ],
      &[
        T(r"\\?\UNC\"),
        U(LOW),
        T("server"),
        U(HIGH),
        T(r"\"),
        U(LOW),
        T("share"),
        U(HIGH),
        T(r"\"),
        U(LOW),
        T("leaf"),
        U(HIGH),
        T(r"\tail"),
      ],
    );

    assert_normalization_case(
      "device namespace field",
      PrefixClass::DeviceNs,
      &[T(r"\\.\"), U(LOW), T("PIPE"), U(HIGH), T(r"\"), U(LOW), T("leaf"), U(HIGH), T(r"\.\tail")],
      &[T(r"\\.\"), U(LOW), T("PIPE"), U(HIGH), T(r"\"), U(LOW), T("leaf"), U(HIGH), T(r"\tail")],
    );

    assert_normalization_case(
      "generic verbatim field",
      PrefixClass::Verbatim,
      &[
        T(r"\\?\"),
        U(LOW),
        T("Volume"),
        U(HIGH),
        T(r"\"),
        U(LOW),
        T("leaf"),
        U(HIGH),
        T(r"\.\tail"),
      ],
      &[T(r"\\?\"), U(LOW), T("Volume"), U(HIGH), T(r"\"), U(LOW), T("leaf"), U(HIGH), T(r"\tail")],
    );

    assert_normalization_case(
      "invalid prefix-only collapse",
      PrefixClass::Verbatim,
      &[
        T(r"\\?\"),
        U(LOW),
        T("namespace"),
        U(HIGH),
        T(r"\"),
        U(LOW),
        T("gone"),
        U(HIGH),
        T(r"\.."),
      ],
      &[T(r"\\?\"), U(LOW), T("namespace"), U(HIGH)],
    );

    assert_normalization_case(
      "invalid prefix-only trailing separator",
      PrefixClass::Verbatim,
      &[
        T(r"\\?\"),
        U(LOW),
        T("namespace"),
        U(HIGH),
        T(r"\"),
        U(LOW),
        T("gone"),
        U(HIGH),
        T(r"\..\"),
      ],
      &[T(r"\\?\"), U(LOW), T("namespace"), U(HIGH), T(r"\")],
    );
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
  fn drive_relative_common_prefix_compares_invalid_encoding_exactly() {
    let same_base = invalid_path(r"C:..\segment-", r"\chunks");
    let same_target = invalid_path(r"c:..\segment-", r"\assets");
    let same_expected = r"..\assets".encode_utf16().collect::<Vec<_>>();
    assert_wide(&same_target.relative(&same_base), &same_expected);
    assert_wide(
      &same_target
        .try_relative(&same_base)
        .expect("matching drive-relative contexts do not need cwd"),
      &same_expected,
    );
    assert_wide(&same_target.relative_with(&same_base, "not/absolute"), &same_expected);

    let base = invalid_path(r"C:..\segment-", r"\chunks");
    let target = invalid_path_with_unit(r"c:..\segment-", DISTINCT_HIGH_SURROGATE, r"\assets");
    let expected = wide_with_invalid_unit(r"..\..\segment-", DISTINCT_HIGH_SURROGATE, r"\assets");

    assert_wide(&target.relative(&base), &expected);
    assert_wide(
      &target.try_relative(&base).expect("matching drive-relative contexts do not need cwd"),
      &expected,
    );
    assert_wide(&target.relative_with(&base, "not/absolute"), &expected);
  }

  #[test]
  fn absolute_relative_common_prefix_compares_invalid_encoding_exactly() {
    let same_base = invalid_path(r"C:\workspace\segment-", r"\chunks");
    let same_target = invalid_path(r"C:\workspace\segment-", r"\assets");
    assert_relative_variants(
      &same_target,
      &same_base,
      &r"..\assets".encode_utf16().collect::<Vec<_>>(),
    );

    assert_eq!(
      invalid_path("segment-", "").to_string_lossy(),
      invalid_path_with_unit("segment-", DISTINCT_HIGH_SURROGATE, "").to_string_lossy(),
    );
    let distinct_base = invalid_path(r"C:\workspace\segment-", r"\chunks");
    let distinct_target =
      invalid_path_with_unit(r"C:\workspace\segment-", DISTINCT_HIGH_SURROGATE, r"\assets");
    let expected = wide_with_invalid_unit(r"..\..\segment-", DISTINCT_HIGH_SURROGATE, r"\assets");
    assert_relative_variants(&distinct_target, &distinct_base, &expected);
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

  #[test]
  fn drive_relative_absolutize_with_preserves_invalid_wide_encoding() {
    let input = invalid_path(r"C:pkg\invalid-", r"\.\file");
    let expected = wide_with_invalid(r"C:\workspace\pkg\invalid-", r"\file");

    let borrowed_cwd = input.absolutize_with(Path::new(r"C:\workspace"));
    assert_wide(&borrowed_cwd, &expected);
    assert!(matches!(borrowed_cwd, Cow::Owned(_)));

    let owned_cwd = input.absolutize_with(PathBuf::from(r"C:\workspace"));
    assert_wide(&owned_cwd, &expected);
    assert!(matches!(owned_cwd, Cow::Owned(_)));
  }
}
