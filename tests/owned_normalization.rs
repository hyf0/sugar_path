use std::path::{Path, PathBuf};

use sugar_path::SugarPathBuf;

fn assert_owned_normalization(case: &str, input: &str, expected: &str) {
  let normalized = PathBuf::from(input).into_normalized();
  assert_eq!(
    normalized.as_os_str(),
    Path::new(expected).as_os_str(),
    "unexpected owned normalization for {case}: {input:?}",
  );

  let normalized_twice = normalized.clone().into_normalized();
  assert_eq!(
    normalized_twice.as_os_str(),
    normalized.as_os_str(),
    "owned normalization was not exactly idempotent for {case}: {input:?}",
  );
}

#[cfg(target_family = "unix")]
#[test]
fn unix_owned_normalization_has_exact_independent_oracles() {
  let cases = [
    ("empty", "", "."),
    ("current directory with redundant separator", ".//", "./"),
    ("root only", "/", "/"),
    ("redundant root separators", "////", "/"),
    ("absolute root clamp", "/a/../../tail/", "/tail/"),
    ("clean relative", "foo/bar", "foo/bar"),
    ("clean relative trailing separator", "foo/bar/", "foo/bar/"),
    ("dirty relative trailing separator", "foo//./bar/../baz//", "foo/baz/"),
    ("collapse to current directory", "foo/..", "."),
    ("collapse to current directory with trailing separator", "foo/../", "./"),
    ("unresolved leading parents", "../../../foo/../bar/", "../../../bar/"),
    ("backslash is a normal character", r"foo\bar/./baz", r"foo\bar/baz"),
    ("valid non-ASCII components", "/café/./文件/", "/café/文件/"),
    (
      "dirty path beyond the inline component capacity",
      "a/b/c/d/e/f/g/h/i/../j/",
      "a/b/c/d/e/f/g/h/j/",
    ),
  ];

  for (case, input, expected) in cases {
    assert_owned_normalization(case, input, expected);
  }
}

#[cfg(target_family = "unix")]
#[test]
fn non_utf8_unix_owned_normalization_preserves_raw_bytes() {
  use std::{
    ffi::OsString,
    os::unix::ffi::{OsStrExt, OsStringExt},
  };

  let cases: [(&str, &[u8], &[u8]); 6] = [
    ("clean relative component", b"dir/invalid-\x80/file", b"dir/invalid-\x80/file"),
    (
      "clean absolute component with trailing separator",
      b"/dir/invalid-\xff/file/",
      b"/dir/invalid-\xff/file/",
    ),
    ("dirty relative component", b"dir/invalid-\x80/./file", b"dir/invalid-\x80/file"),
    (
      "unresolved leading parents and trailing separator",
      b"../../invalid-\xff/./file//",
      b"../../invalid-\xff/file/",
    ),
    ("absolute root clamp", b"/../../invalid-\x80/./file/", b"/invalid-\x80/file/"),
    ("invalid component removed by a parent", b"dir/invalid-\xff/../", b"dir/"),
  ];

  for (case, input, expected) in cases {
    let normalized = PathBuf::from(OsString::from_vec(input.to_vec())).into_normalized();
    assert_eq!(
      normalized.as_os_str().as_bytes(),
      expected,
      "unexpected owned normalization for {case}",
    );

    let normalized_twice = normalized.clone().into_normalized();
    assert_eq!(
      normalized_twice.as_os_str().as_bytes(),
      expected,
      "invalid Unix encoding was not exactly idempotent for {case}",
    );
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_owned_normalization_has_exact_independent_oracles() {
  let cases = [
    ("empty", "", "."),
    ("current directory with redundant separator", r".\\", r".\"),
    ("root only", r"\", r"\"),
    ("root-relative", r"\foo\..\bar\\", r"\bar\"),
    ("root-relative clamp", r"\..\workspace\pkg\..\dist\", r"\workspace\dist\"),
    ("clean relative trailing separator", r"foo\bar\", r"foo\bar\"),
    ("dirty relative trailing separator", r"foo\\.\bar\..\baz\\", r"foo\baz\"),
    ("unresolved leading parents", r"..\..\pkg\.\src\..\dist\", r"..\..\pkg\dist\"),
    ("disk prefix only", r"C:", r"C:."),
    ("disk root", r"c:\", r"c:\"),
    ("ordinary disk with mixed separators", r"c:/foo//./bar/../", r"c:\foo\"),
    ("ordinary disk root clamp", r"C:\..\..\foo\", r"C:\foo\"),
    ("drive-relative", r"C:foo\..\bar\", r"C:bar\"),
    ("drive-relative unresolved parent", r"c:..\foo\..\bar", r"c:..\bar"),
    ("drive-relative collapse with trailing separator", r"C:pkg\..\", r"C:.\"),
    ("ordinary UNC prefix only", r"\\server\share", r"\\server\share\"),
    (
      "ordinary UNC root clamp with trailing separator",
      r"\\server\share\pkg\..\..\file\",
      r"\\server\share\file\",
    ),
    ("verbatim disk prefix only", r"\\?\C:", r"\\?\C:"),
    ("verbatim disk dirty trailing path", r"\\?\c:\dir\..\file\", r"\\?\c:\file\"),
    ("verbatim disk rooted collapse", r"\\?\C:\pkg\..", r"\\?\C:\"),
    ("verbatim disk literal forward slash", r"\\?\C:\foo/", r"\\?\C:\foo/"),
    ("verbatim disk dirty path with literal forward slash", r"\\?\C:\dir\..\foo/", r"\\?\C:\foo/"),
    ("verbatim UNC prefix only", r"\\?\UNC\server\share", r"\\?\UNC\server\share"),
    (
      "verbatim UNC dirty trailing path",
      r"\\?\UNC\server\share\dir\..\file\",
      r"\\?\UNC\server\share\file\",
    ),
    (
      "verbatim UNC literal forward slash",
      r"\\?\UNC\server\share\pkg\src\..\dist/file",
      r"\\?\UNC\server\share\pkg\dist/file",
    ),
    (
      "verbatim UNC collapse without trailing separator",
      r"\\?\UNC\server\share\pkg\..",
      r"\\?\UNC\server\share",
    ),
    (
      "verbatim UNC collapse with trailing separator",
      r"\\?\UNC\server\share\pkg\..\",
      r"\\?\UNC\server\share\",
    ),
    ("device namespace prefix only", r"\\.\PIPE", r"\\.\PIPE"),
    ("device namespace dirty trailing path", r"\\.\PIPE\dir\..\file\", r"\\.\PIPE\file\"),
    ("device namespace collapse without trailing separator", r"\\.\PIPE\pkg\..", r"\\.\PIPE"),
    ("device namespace collapse with trailing separator", r"\\.\PIPE\pkg\..\", r"\\.\PIPE\"),
    ("generic verbatim prefix only", r"\\?\Volume{abc}", r"\\?\Volume{abc}"),
    (
      "generic verbatim dirty trailing path",
      r"\\?\Volume{abc}\dir\..\file\",
      r"\\?\Volume{abc}\file\",
    ),
    (
      "generic verbatim literal forward slash",
      r"\\?\Volume{abc}\pkg\src\..\dist/file",
      r"\\?\Volume{abc}\pkg\dist/file",
    ),
    (
      "generic verbatim collapse without trailing separator",
      r"\\?\Volume{abc}\pkg\..",
      r"\\?\Volume{abc}",
    ),
    (
      "generic verbatim collapse with trailing separator",
      r"\\?\Volume{abc}\pkg\..\",
      r"\\?\Volume{abc}\",
    ),
    ("component must not become a disk prefix", r"dir\..\C:foo", r".\C:foo"),
    ("prefix-looking component remains normal", r"dir\..\C:", r".\C:"),
    ("valid non-ASCII components", r"C:\café\.\文件\", r"C:\café\文件\"),
    (
      "dirty path beyond the inline component capacity",
      r"a\b\c\d\e\f\g\h\i\..\j\",
      r"a\b\c\d\e\f\g\h\j\",
    ),
  ];

  for (case, input, expected) in cases {
    assert_owned_normalization(case, input, expected);
  }
}

#[cfg(target_family = "windows")]
#[test]
fn non_utf8_windows_owned_normalization_preserves_wide_units() {
  use std::{
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
  };

  fn with_invalid_unit(prefix: &str, suffix: &str, invalid: u16) -> Vec<u16> {
    let mut wide = prefix.encode_utf16().collect::<Vec<_>>();
    wide.push(invalid);
    wide.extend(suffix.encode_utf16());
    wide
  }

  let cases = [
    (
      "clean relative component",
      with_invalid_unit(r"dir\invalid-", r"\file", 0xd800),
      with_invalid_unit(r"dir\invalid-", r"\file", 0xd800),
    ),
    (
      "dirty relative component",
      with_invalid_unit(r"dir\invalid-", r"\.\file", 0xd801),
      with_invalid_unit(r"dir\invalid-", r"\file", 0xd801),
    ),
    (
      "clean ordinary disk component with trailing separator",
      with_invalid_unit(r"C:\invalid-", r"\file\", 0xd800),
      with_invalid_unit(r"C:\invalid-", r"\file\", 0xd800),
    ),
    (
      "unresolved leading parent and trailing separator",
      with_invalid_unit(r"..\invalid-", r"\.\file\\", 0xd800),
      with_invalid_unit(r"..\invalid-", r"\file\", 0xd800),
    ),
    (
      "ordinary disk root clamp",
      with_invalid_unit(r"C:\..\invalid-", r"\.\file\", 0xd801),
      with_invalid_unit(r"C:\invalid-", r"\file\", 0xd801),
    ),
    (
      "ordinary UNC server identifier",
      with_invalid_unit(r"\\server-", r"\share\base\.\child\..\file\", 0xd800),
      with_invalid_unit(r"\\server-", r"\share\base\file\", 0xd800),
    ),
    (
      "ordinary UNC share identifier",
      with_invalid_unit(r"\\server\share-", r"\base\.\child\..\file\", 0xd801),
      with_invalid_unit(r"\\server\share-", r"\base\file\", 0xd801),
    ),
    (
      "verbatim UNC server identifier",
      with_invalid_unit(r"\\?\UNC\server-", r"\share\base\.\child\..\file\", 0xd800),
      with_invalid_unit(r"\\?\UNC\server-", r"\share\base\file\", 0xd800),
    ),
    (
      "verbatim UNC share identifier",
      with_invalid_unit(r"\\?\UNC\server\share-", r"\base\.\child\..\file\", 0xd801),
      with_invalid_unit(r"\\?\UNC\server\share-", r"\base\file\", 0xd801),
    ),
    (
      "device namespace identifier",
      with_invalid_unit(r"\\.\PIPE-", r"\base\.\child\..\file\", 0xd800),
      with_invalid_unit(r"\\.\PIPE-", r"\base\file\", 0xd800),
    ),
    (
      "generic verbatim identifier",
      with_invalid_unit(r"\\?\Volume-", r"\base\.\child\..\file\", 0xd801),
      with_invalid_unit(r"\\?\Volume-", r"\base\file\", 0xd801),
    ),
  ];

  for (case, input, expected) in cases {
    let normalized = PathBuf::from(OsString::from_wide(&input)).into_normalized();
    assert_eq!(
      normalized.as_os_str().encode_wide().collect::<Vec<_>>(),
      expected,
      "unexpected owned normalization for {case}",
    );

    let normalized_twice = normalized.into_normalized();
    assert_eq!(
      normalized_twice.as_os_str().encode_wide().collect::<Vec<_>>(),
      expected,
      "invalid Windows encoding was not exactly idempotent for {case}",
    );
  }
}
