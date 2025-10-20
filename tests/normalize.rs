use sugar_path::SugarPath;
mod test_utils;

#[cfg(target_family = "unix")]
#[test]
fn unix() {
  assert_eq!(p!("/foo/../../../bar").normalize(), p!("/bar"));
  assert_eq!(p!("a//b//../b").normalize(), p!("a/b"));
  assert_eq!(p!("/foo/../../../bar").normalize(), p!("/bar"));
  assert_eq!(p!("a//b//./c").normalize(), p!("a/b/c"));
  assert_eq!(p!("a//b//.").normalize(), p!("a/b"));
  assert_eq!(p!("/a/b/c/../../../x/y/z").normalize(), p!("/x/y/z"));
  assert_eq!(p!("///..//./foo/.//bar").normalize(), p!("/foo/bar"));
  assert_eq!(p!("bar/foo../../").normalize(), p!("bar/"));
  assert_eq!(p!("bar/foo../..").normalize(), p!("bar"));
  assert_eq!(p!("bar/foo../../baz").normalize(), p!("bar/baz"));
  assert_eq!(p!("bar/foo../").normalize(), p!("bar/foo../"));
  assert_eq!(p!("bar/foo..").normalize(), p!("bar/foo.."));
  assert_eq!(p!("../foo../../../bar").normalize(), p!("../../bar"));
  assert_eq!(p!("../foo../../../bar").normalize(), p!("../../bar"));
  assert_eq!(p!("../.../.././.../../../bar").normalize(), p!("../../bar"));
  assert_eq!(p!("../.../.././.../../../bar").normalize(), p!("../../bar"));
  assert_eq!(p!("../../../foo/../../../bar").normalize(), p!("../../../../../bar"));
  assert_eq!(p!("../../../foo/../../../bar/../../").normalize(), p!("../../../../../../"));
  assert_eq!(p!("../foobar/barfoo/foo/../../../bar/../../").normalize(), p!("../../"));
  assert_eq!(p!("../.../../foobar/../../../bar/../../baz").normalize(), p!("../../../../baz"));
  assert_eq!(p!("foo/bar\\baz").normalize(), p!("foo/bar\\baz"));
  assert_eq!(p!("/a/b/c/../../../").normalize(), p!("/"));
  assert_eq!(p!("a/b/c/../../../").normalize(), p!("."));
  assert_eq!(p!("a/b/c/../../..").normalize(), p!("."));

  assert_eq!(p!("").normalize(), p!("."));
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
  assert_eq!(p!("").normalize(), p!("."));
  assert_eq!(p!("./fixtures///b/../b/c.js").normalize(), p!("fixtures\\b\\c.js"));
  assert_eq!(p!("/foo/../../../bar").normalize(), p!("\\bar"));
  assert_eq!(p!("a//b//../b").normalize(), p!("a\\b"));
  assert_eq!(p!("a//b//./c").normalize(), p!("a\\b\\c"));
  assert_eq!(p!("//server/share/dir/file.ext").normalize(), p!("\\\\server\\share\\dir\\file.ext"));
  assert_eq!(p!("/foo/../../../bar").normalize(), p!("\\bar"));
  assert_eq!(p!("/a/b/c/../../../x/y/z").normalize(), p!("\\x\\y\\z"));
  assert_eq!(p!("C:").normalize(), p!("C:."));
  assert_eq!(p!("C:/").normalize(), p!("C:\\"));
  assert_eq!(p!("").normalize(), p!("."));
  assert_eq!(p!("c:/ignore").normalize(), p!("c:\\ignore"));
  assert_eq!(p!("C:../a").normalize(), p!("C:..\\a"));
  assert_eq!(p!("c:/../a").normalize(), p!("c:\\a"));
  assert_eq!(p!("C:..\\..\\abc\\..\\def").normalize(), p!("C:..\\..\\def"));
  assert_eq!(p!("C:\\..\\..\\abc\\..\\def").normalize(), p!("C:\\def"));
  assert_eq!(p!("C:\\.").normalize(), p!("C:\\"));

  assert_eq!(p!("file:stream").normalize(), p!("file:stream"));
  assert_eq!(p!("bar\\foo..\\..\\").normalize(), p!("bar\\"));
  assert_eq!(p!("bar\\foo..\\..\\").normalize(), p!("bar\\"));
  assert_eq!(p!("bar\\foo..\\..").normalize(), p!("bar"));
  assert_eq!(p!("bar\\foo..\\..\\baz").normalize(), p!("bar\\baz"));
  assert_eq!(p!("bar\\foo..\\").normalize(), p!("bar\\foo..\\"));
  assert_eq!(p!("..\\foo..\\..\\..\\bar").normalize(), p!("..\\..\\bar"));
  assert_eq!(p!("..\\...\\..\\.\\...\\..\\..\\bar").normalize(), p!("..\\..\\bar"));
  assert_eq!(p!("../../../foo/../../../bar").normalize(), p!("..\\..\\..\\..\\..\\bar"));
  assert_eq!(p!("../../../foo/../../../bar/../../").normalize(), p!("..\\..\\..\\..\\..\\..\\"));
  assert_eq!(p!("../foobar/barfoo/foo/../../../bar/../../").normalize(), p!("..\\..\\"));
  assert_eq!(p!("../.../../foobar/../../../bar/../../baz").normalize(), p!("..\\..\\..\\..\\baz"));
  assert_eq!(p!("foo/bar\\baz").normalize(), p!("foo\\bar\\baz"));
}
