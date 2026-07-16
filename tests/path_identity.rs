#![cfg(any(unix, windows))]

use std::path::{Path, PathBuf};

use sugar_path::{SugarPath, SugarPathBuf};

fn assert_exact_normalization(input: &str, expected: &str) {
  let expected = Path::new(expected);
  let normalized = Path::new(input).normalize();
  assert_eq!(
    normalized.as_os_str(),
    expected.as_os_str(),
    "normalizing {input:?} produced the wrong exact spelling",
  );
  assert_eq!(
    normalized.normalize().as_os_str(),
    normalized.as_os_str(),
    "normalizing {input:?} a second time changed its representation",
  );

  let consumed = PathBuf::from(input).into_normalized();
  assert_eq!(
    consumed.as_os_str(),
    expected.as_os_str(),
    "consuming normalization of {input:?} produced the wrong exact spelling",
  );
  assert_eq!(
    consumed.clone().into_normalized().as_os_str(),
    consumed.as_os_str(),
    "consuming normalization of {input:?} was not exactly idempotent",
  );
}

fn assert_path_equal_spellings_remain_distinct(left: &str, right: &str) {
  let left_path = Path::new(left);
  let right_path = Path::new(right);
  assert_eq!(left_path, right_path, "the fixture must compare equal as standard Path values");
  assert_ne!(
    left_path.as_os_str(),
    right_path.as_os_str(),
    "the fixture must retain distinct native spellings",
  );
  assert_exact_normalization(left, left);
  assert_exact_normalization(right, right);
}

fn assert_normalization_is_idempotent(paths: &[&str]) {
  for path in paths {
    let once = Path::new(path).normalize().into_owned();
    let twice = once.normalize();
    assert_eq!(
      twice.as_os_str(),
      once.as_os_str(),
      "normalizing {path:?} a second time changed its representation",
    );
  }
}

#[cfg(unix)]
const NATIVE_CORPUS: &[&str] = &[
  "",
  ".",
  "./",
  ".//",
  "./.",
  "foo",
  "foo/",
  "foo//",
  "./foo",
  "foo/.",
  "foo/./",
  "foo/bar",
  "foo//bar",
  "foo///bar/",
  "foo/bar/..",
  "foo/bar/../",
  "foo/../../bar",
  "../",
  "../foo/",
  "../foo/../bar",
  "/",
  "//",
  "/./",
  "/foo/",
  "/foo/bar/..",
];

#[cfg(windows)]
const NATIVE_CORPUS: &[&str] = &[
  "",
  ".",
  r".\",
  r".\\",
  "foo",
  r"foo\",
  r"foo\\",
  r"foo\.\",
  r"foo\bar\..\",
  r"..\",
  r"..\foo\",
  r"\",
  r"\foo\",
  r"C:",
  r"c:",
  r"C:\",
  r"c:\workspace\pkg\",
  r"C:workspace\pkg\",
  r"\\server\share",
  r"\\server\share\dir\",
  r"\\?\c:\workspace\pkg\",
  r"\\?\UNC\server\share\dir\",
  r"\\.\PIPE\rolldown\dir\",
  r"\\?\Volume{abc}\dir\",
];

#[test]
fn normalization_is_exactly_idempotent() {
  assert_normalization_is_idempotent(NATIVE_CORPUS);
}

#[cfg(unix)]
#[test]
fn unix_node_style_trailing_spelling_is_exact() {
  for (input, expected) in [
    ("", "."),
    (".", "."),
    ("./", "./"),
    (".//", "./"),
    ("foo/", "foo/"),
    ("foo//", "foo/"),
    ("foo/.", "foo"),
    ("foo/./", "foo/"),
    ("foo/..", "."),
    ("foo/../", "./"),
    ("../", "../"),
    ("/foo/../", "/"),
  ] {
    assert_eq!(Path::new(input).normalize().as_os_str(), Path::new(expected).as_os_str());
  }
}

#[cfg(unix)]
#[test]
fn unix_path_equality_does_not_define_normalized_spelling() {
  for (plain, trailing) in [(".", "./"), ("foo", "foo/")] {
    assert_path_equal_spellings_remain_distinct(plain, trailing);
  }
}

#[cfg(windows)]
#[test]
fn windows_node_style_trailing_and_drive_spelling_is_exact() {
  for (input, expected) in [
    ("", "."),
    (".", "."),
    (r".\", r".\"),
    (r".\\", r".\"),
    (r"foo\", r"foo\"),
    (r"foo\\", r"foo\"),
    (r"foo\.\", r"foo\"),
    (r"foo\..", "."),
    (r"foo\..\", r".\"),
    (r"..\", r"..\"),
    (r"C:", r"C:."),
    (r"c:", r"c:."),
    (r"c:\foo\", r"c:\foo\"),
    (r"\\?\c:\foo\", r"\\?\c:\foo\"),
  ] {
    assert_eq!(Path::new(input).normalize().as_os_str(), Path::new(expected).as_os_str());
  }
}

#[cfg(windows)]
#[test]
fn windows_path_equality_does_not_define_normalized_spelling() {
  for (left, right) in [(".", r".\"), ("foo", r"foo\"), (r"C:\foo", r"c:\foo")] {
    assert_path_equal_spellings_remain_distinct(left, right);
  }
}

#[cfg(windows)]
#[test]
fn windows_prefix_like_normal_components_remain_exactly_idempotent() {
  for (input, expected) in [("...:/..", "."), ("..:/../", r".\")] {
    assert_exact_normalization(input, expected);
  }
}
