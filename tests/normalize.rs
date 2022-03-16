use std::path::{Path};
use sugar_path::{PathSugar};

#[cfg(target_family = "unix")]
#[test]
fn unix() {
  assert_eq!(Path::new("/foo/../../../bar").normalize(), Path::new("/bar"));
  assert_eq!(Path::new("a//b//../b").normalize(), Path::new("a/b"));
  assert_eq!(Path::new("/foo/../../../bar").normalize(), Path::new("/bar"));
  assert_eq!(Path::new("a//b//./c").normalize(), Path::new("a/b/c"));
  assert_eq!(Path::new("a//b//.").normalize(), Path::new("a/b"));
  assert_eq!(Path::new("/a/b/c/../../../x/y/z").normalize(), Path::new("/x/y/z"));
  assert_eq!(Path::new("///..//./foo/.//bar").normalize(), Path::new("/foo/bar"));
  assert_eq!(Path::new("bar/foo../../").normalize(), Path::new("bar/"));
  assert_eq!(Path::new("bar/foo../..").normalize(), Path::new("bar"));
  assert_eq!(Path::new("bar/foo../../baz").normalize(), Path::new("bar/baz"));
  assert_eq!(Path::new("bar/foo../").normalize(), Path::new("bar/foo../"));
  assert_eq!(Path::new("bar/foo..").normalize(), Path::new("bar/foo.."));
  assert_eq!(Path::new("../foo../../../bar").normalize(), Path::new("../../bar"));
  assert_eq!(Path::new("../foo../../../bar").normalize(), Path::new("../../bar"));
  assert_eq!(Path::new("../.../.././.../../../bar").normalize(), Path::new("../../bar"));
  assert_eq!(Path::new("../.../.././.../../../bar").normalize(), Path::new("../../bar"));
  assert_eq!(Path::new("../../../foo/../../../bar").normalize(), Path::new("../../../../../bar"));
  assert_eq!(Path::new("../../../foo/../../../bar/../../").normalize(), Path::new("../../../../../../"));
  assert_eq!(Path::new("../foobar/barfoo/foo/../../../bar/../../").normalize(), Path::new("../../"));
  assert_eq!(Path::new("../.../../foobar/../../../bar/../../baz").normalize(), Path::new("../../../../baz"));
  assert_eq!(Path::new("foo/bar\\baz").normalize(), Path::new("foo/bar\\baz"));
  // TODO: how we handle ""
  // assert_eq!(&nodejs_path::posix::normalize(""), ".");
}

#[cfg(target_family = "windows")]
fn windows() {
  assert_eq!(Path::new("./fixtures///b/../b/c.js").normalize(), Path::new("fixtures\\b\\c.js"));
}