use std::path::Path;

use sugar_path::SugarPath;
mod test_utils;

#[derive(Debug)]
pub struct Case {
  input: &'static str,
  expected_path: &'static Path,
  expected_slash: &'static str,
}

#[cfg(target_family = "unix")]
#[test]
fn unix() {
  let cases = [
    ("/foo/../../../bar", "/bar", "/bar"),
    ("a//b//../b", "a/b", "a/b"),
    ("/foo/../../../bar", "/bar", "/bar"),
    ("a//b//./c", "a/b/c", "a/b/c"),
    ("a//b//.", "a/b", "a/b"),
    ("/a/b/c/../../../x/y/z", "/x/y/z", "/x/y/z"),
    ("///..//./foo/.//bar", "/foo/bar", "/foo/bar"),
    ("bar/foo../../", "bar/", "bar"),
    ("bar/foo../..", "bar", "bar"),
    ("bar/foo../../baz", "bar/baz", "bar/baz"),
    ("bar/foo../", "bar/foo../", "bar/foo.."),
    ("bar/foo..", "bar/foo..", "bar/foo.."),
    ("../foo../../../bar", "../../bar", "../../bar"),
    ("../foo../../../bar", "../../bar", "../../bar"),
    ("../.../.././.../../../bar", "../../bar", "../../bar"),
    ("../.../.././.../../../bar", "../../bar", "../../bar"),
    ("../../../foo/../../../bar", "../../../../../bar", "../../../../../bar"),
    ("../../../foo/../../../bar/../../", "../../../../../../", "../../../../../.."),
    ("../foobar/barfoo/foo/../../../bar/../../", "../../", "../.."),
    ("../.../../foobar/../../../bar/../../baz", "../../../../baz", "../../../../baz"),
    ("foo/bar\\baz", "foo/bar\\baz", "foo/bar\\baz"),
    ("/a/b/c/../../../", "/", "/"),
    ("a/b/c/../../../", ".", "."),
    ("a/b/c/../../..", ".", "."),
    ("", ".", "."),
  ]
  .into_iter()
  .map(|item| Case { input: item.0, expected_path: Path::new(item.1), expected_slash: item.2 });

  for case in cases {
    let normalized = case.input.normalize();
    assert_eq!(normalized, case.expected_path, "case: {case:#?}");
    assert_eq!(normalized.to_slash().as_deref(), Some(case.expected_slash), "case: {case:#?}");
    assert_eq!(normalized.to_slash_lossy(), case.expected_slash, "case: {case:#?}");
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
  let cases = [
    ("", ".", "."),
    ("./fixtures///b/../b/c.js", "fixtures\\b\\c.js", "fixtures\\b\\c.js"),
    ("/foo/../../../bar", "\\bar", "/bar"),
    ("a//b//../b", "a\\b", "a/b"),
    ("a//b//./c", "a\\b\\c", "a/b/c"),
    (
      "//server/share/dir/file.ext",
      "\\\\server\\share\\dir\\file.ext",
      "//server/share/dir/file.ext",
    ),
    ("/foo/../../../bar", "\\bar", "/bar"),
    ("/a/b/c/../../../x/y/z", "\\x\\y\\z", "/x/y/z"),
    ("C:", "C:.", "C:."),
    ("C:/", "C:\\", "C:\\"),
    ("", ".", "."),
    ("c:/ignore", "c:\\ignore", "c:/ignore"),
    ("C:../a", "C:..\\a", "C:..\\a"),
    ("c:/../a", "c:\\a", "c:/a"),
    ("C:..\\..\\abc\\..\\def", "C:..\\..\\def", "C:..\\..\\def"),
    ("C:\\..\\..\\abc\\..\\def", "C:\\def", "C:\\def"),
    ("C:\\.", "C:\\", "C:\\"),
    ("file:stream", "file:stream", "file:stream"),
    ("bar\\foo..\\..\\", "bar\\", "bar"),
    ("bar\\foo..\\..\\", "bar\\", "bar"),
    ("bar\\foo..\\..", "bar", "bar"),
    ("bar\\foo..\\..\\baz", "bar\\baz", "bar/baz"),
    ("bar\\foo..\\", "bar\\foo..\\", "bar/foo.."),
    ("..\\foo..\\..\\..\\bar", "..\\..\\bar", "../../bar"),
    ("..\\...\\..\\.\\...\\..\\..\\bar", "..\\..\\bar", "../../bar"),
    ("../../../foo/../../../bar", "..\\..\\..\\..\\..\\bar", "../../../../../bar"),
    ("../../../foo/../../../bar/../../", "..\\..\\..\\..\\..\\..\\", "../../../../../../"),
    ("../foobar/barfoo/foo/../../../bar/../../", "..\\..\\", "../.."),
    ("../.../../foobar/../../../bar/../../baz", "..\\..\\..\\..\\baz", "../../../../baz"),
    ("foo/bar\\baz", "foo\\bar\\baz", "foo/bar\\baz"),
  ]
  .into_iter()
  .map(|item| Case { input: item.0, expected_path: Path::new(item.1), expected_slash: item.2 });

  for case in cases {
    let normalized = case.input.normalize();
    assert_eq!(normalized, case.expected_path, "case: {case:#?}");
    assert_eq!(normalized.to_slash().as_deref(), Some(case.expected_slash), "case: {case:#?}");
    assert_eq!(normalized.to_slash_lossy(), case.expected_slash, "case: {case:#?}");
  }
}
