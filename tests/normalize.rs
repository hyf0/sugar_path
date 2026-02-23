use sugar_path::SugarPath;
mod test_utils;

#[cfg(target_family = "unix")]
#[test]
fn unix() {
  assert_eq_str!(p!("/foo/../../../bar").normalize(), "/bar");
  assert_eq_str!(p!("a//b//../b").normalize(), "a/b");
  assert_eq_str!(p!("/foo/../../../bar").normalize(), "/bar");
  assert_eq_str!(p!("a//b//./c").normalize(), "a/b/c");
  assert_eq_str!(p!("a//b//.").normalize(), "a/b");
  assert_eq_str!(p!("/a/b/c/../../../x/y/z").normalize(), "/x/y/z");
  assert_eq_str!(p!("///..//./foo/.//bar").normalize(), "/foo/bar");
  assert_eq_str!(p!("bar/foo../../").normalize(), "bar");
  assert_eq_str!(p!("bar/foo../..").normalize(), "bar");
  assert_eq_str!(p!("bar/foo../../baz").normalize(), "bar/baz");
  assert_eq_str!(p!("bar/foo../").normalize(), "bar/foo..");
  assert_eq_str!(p!("bar/foo..").normalize(), "bar/foo..");
  assert_eq_str!(p!("../foo../../../bar").normalize(), "../../bar");
  assert_eq_str!(p!("../foo../../../bar").normalize(), "../../bar");
  assert_eq_str!(p!("../.../.././.../../../bar").normalize(), "../../bar");
  assert_eq_str!(p!("../.../.././.../../../bar").normalize(), "../../bar");
  assert_eq_str!(p!("../../../foo/../../../bar").normalize(), "../../../../../bar");
  assert_eq_str!(p!("../../../foo/../../../bar/../../").normalize(), "../../../../../..");
  assert_eq_str!(p!("../foobar/barfoo/foo/../../../bar/../../").normalize(), "../..");
  assert_eq_str!(p!("../.../../foobar/../../../bar/../../baz").normalize(), "../../../../baz");
  assert_eq_str!(p!("foo/bar\\baz").normalize(), "foo/bar\\baz");
  assert_eq_str!(p!("/a/b/c/../../../").normalize(), "/");
  assert_eq_str!(p!("a/b/c/../../../").normalize(), ".");
  assert_eq_str!(p!("a/b/c/../../..").normalize(), ".");

  assert_eq_str!(p!("").normalize(), ".");
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
  assert_eq_str!(p!("").normalize(), ".");
  assert_eq_str!(p!("./fixtures///b/../b/c.js").normalize(), "fixtures\\b\\c.js");
  assert_eq_str!(p!("/foo/../../../bar").normalize(), "\\bar");
  assert_eq_str!(p!("a//b//../b").normalize(), "a\\b");
  assert_eq_str!(p!("a//b//./c").normalize(), "a\\b\\c");
  assert_eq_str!(p!("//server/share/dir/file.ext").normalize(), "\\\\server\\share\\dir\\file.ext");
  assert_eq_str!(p!("/foo/../../../bar").normalize(), "\\bar");
  assert_eq_str!(p!("/a/b/c/../../../x/y/z").normalize(), "\\x\\y\\z");
  assert_eq_str!(p!("C:").normalize(), "C:.");
  assert_eq_str!(p!("C:/").normalize(), "C:\\");
  assert_eq_str!(p!("").normalize(), ".");
  assert_eq_str!(p!("c:/ignore").normalize(), "c:\\ignore");
  assert_eq_str!(p!("C:../a").normalize(), "C:..\\a");
  assert_eq_str!(p!("c:/../a").normalize(), "c:\\a");
  assert_eq_str!(p!("C:..\\..\\abc\\..\\def").normalize(), "C:..\\..\\def");
  assert_eq_str!(p!("C:\\..\\..\\abc\\..\\def").normalize(), "C:\\def");
  assert_eq_str!(p!("C:\\.").normalize(), "C:\\");

  assert_eq_str!(p!("file:stream").normalize(), "file:stream");
  assert_eq_str!(p!("bar\\foo..\\..\\").normalize(), "bar");
  assert_eq_str!(p!("bar\\foo..\\..\\").normalize(), "bar");
  assert_eq_str!(p!("bar\\foo..\\..").normalize(), "bar");
  assert_eq_str!(p!("bar\\foo..\\..\\baz").normalize(), "bar\\baz");
  assert_eq_str!(p!("bar\\foo..\\").normalize(), "bar\\foo..");
  assert_eq_str!(p!("..\\foo..\\..\\..\\bar").normalize(), "..\\..\\bar");
  assert_eq_str!(p!("..\\...\\..\\.\\...\\..\\..\\bar").normalize(), "..\\..\\bar");
  assert_eq_str!(p!("../../../foo/../../../bar").normalize(), "..\\..\\..\\..\\..\\bar");
  assert_eq_str!(p!("../../../foo/../../../bar/../../").normalize(), "..\\..\\..\\..\\..\\..");
  assert_eq_str!(p!("../foobar/barfoo/foo/../../../bar/../../").normalize(), "..\\..");
  assert_eq_str!(p!("../.../../foobar/../../../bar/../../baz").normalize(), "..\\..\\..\\..\\baz");
  assert_eq_str!(p!("foo/bar\\baz").normalize(), "foo\\bar\\baz");
}
