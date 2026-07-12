#![cfg(any(unix, windows))]

use std::path::Path;

use sugar_path::SugarPath;

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
fn path_equal_trailing_spellings_may_normalize_differently() {
  let plain = Path::new("foo");
  let trailing = Path::new("foo/");
  assert_eq!(plain, trailing);
  assert_eq!(plain.normalize().as_os_str(), Path::new("foo").as_os_str());
  assert_eq!(trailing.normalize().as_os_str(), Path::new("foo/").as_os_str());
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
