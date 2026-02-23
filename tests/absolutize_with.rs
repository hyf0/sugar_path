#[cfg(not(windows))]
use std::path::PathBuf;
use sugar_path::SugarPath;
mod test_utils;

#[cfg(target_family = "unix")]
#[test]
fn unix_absolutize_with() {
  // Basic absolutize_with tests
  assert_eq_str!("./world".absolutize_with("/hello"), "/hello/world");
  assert_eq_str!("../world".absolutize_with("/hello"), "/world");
  assert_eq_str!("world".absolutize_with("/hello"), "/hello/world");

  // With absolute paths as input
  assert_eq_str!("/absolute".absolutize_with("/base"), "/absolute");
  assert_eq_str!("/usr/bin".absolutize_with("/home"), "/usr/bin");

  // With dots in paths
  assert_eq_str!("./a/./b/../c".absolutize_with("/base"), "/base/a/c");
  assert_eq_str!("../a/../b".absolutize_with("/base/dir"), "/base/b");

  // Empty path
  assert_eq_str!("".absolutize_with("/base"), "/base");
  assert_eq_str!(".".absolutize_with("/base"), "/base");

  // Multiple levels up
  assert_eq_str!("../../file".absolutize_with("/a/b/c"), "/a/file");
  assert_eq_str!("../../../file".absolutize_with("/a/b/c"), "/file");

  // Complex paths
  assert_eq_str!("./foo/../bar/./baz".absolutize_with("/root"), "/root/bar/baz");
  assert_eq_str!("a/b/../../c".absolutize_with("/base"), "/base/c");
}

#[cfg(target_family = "unix")]
#[test]
fn unix_absolutize_with_trailing_slash() {
  // Test with trailing slashes
  assert_eq_str!("world/".absolutize_with("/hello/"), "/hello/world");
  assert_eq_str!("./world/".absolutize_with("/hello"), "/hello/world");
}

#[cfg(target_family = "unix")]
#[test]
fn unix_absolutize_with_string_types() {
  // Test with different string types as base
  let base = String::from("/home/user");
  assert_eq_str!("documents".absolutize_with(&base), "/home/user/documents");
  assert_eq_str!("../downloads".absolutize_with(base.as_str()), "/home/downloads");

  // Test with PathBuf as base
  let base_path = PathBuf::from("/var/log");
  assert_eq_str!("app.log".absolutize_with(&base_path), "/var/log/app.log");
}

#[cfg(target_family = "windows")]
#[test]
fn windows_absolutize_with() {
  // Basic absolutize_with tests
  assert_eq_str!(".\\world".absolutize_with("C:\\hello"), "C:\\hello\\world");
  assert_eq_str!("..\\world".absolutize_with("C:\\hello"), "C:\\world");
  assert_eq_str!("world".absolutize_with("C:\\hello"), "C:\\hello\\world");

  // With absolute paths as input
  assert_eq_str!("D:\\absolute".absolutize_with("C:\\base"), "D:\\absolute");
  assert_eq_str!("C:\\Windows".absolutize_with("C:\\Users"), "C:\\Windows");

  // With dots in paths
  assert_eq_str!(".\\a\\.\\b\\..\\c".absolutize_with("C:\\base"), "C:\\base\\a\\c");
  assert_eq_str!("..\\a\\..\\b".absolutize_with("C:\\base\\dir"), "C:\\base\\b");

  // Empty path
  assert_eq_str!("".absolutize_with("C:\\base"), "C:\\base");
  assert_eq_str!(".".absolutize_with("C:\\base"), "C:\\base");

  // Multiple levels up
  assert_eq_str!("..\\..\\file".absolutize_with("C:\\a\\b\\c"), "C:\\a\\file");

  // Drive-relative paths
  assert_eq_str!("C:file".absolutize_with("D:\\base"), "C:\\file");
  assert_eq_str!("C:.\\file".absolutize_with("D:\\base"), "C:\\file");
}

#[cfg(target_family = "windows")]
#[test]
fn windows_absolutize_with_unc_paths() {
  // UNC path tests
  assert_eq_str!("file".absolutize_with("\\\\server\\share"), "\\\\server\\share\\file");
  assert_eq_str!(
    "..\\other".absolutize_with("\\\\server\\share\\folder"),
    "\\\\server\\share\\other"
  );
  assert_eq_str!("\\\\other\\share".absolutize_with("\\\\server\\share"), "\\\\other\\share\\");
}

#[cfg(target_family = "windows")]
#[test]
fn windows_absolutize_with_mixed_separators() {
  // Test with mixed separators
  assert_eq_str!("sub/folder".absolutize_with("C:\\base"), "C:\\base\\sub\\folder");
  assert_eq_str!("./sub\\folder".absolutize_with("C:/base"), "C:\\base\\sub\\folder");
}

#[test]
fn absolutize_with_relative_base() {
  // When base is relative, absolutize_with will resolve to absolute paths
  // based on current working directory
  let cwd = std::env::current_dir().unwrap();
  assert_eq!("file".absolutize_with("relative/path"), cwd.join("relative/path/file"));
  assert_eq!("./file".absolutize_with("./base"), cwd.join("base/file"));
  assert_eq!("../file".absolutize_with("base/dir"), cwd.join("base/file"));
}

#[test]
fn absolutize_with_edge_cases() {
  // Edge cases - absolutize_with always produces absolute paths
  let cwd = std::env::current_dir().unwrap();
  assert_eq!("..".absolutize_with("base"), cwd);

  // Going up two levels from base/dir should end up at cwd
  assert_eq!("../..".absolutize_with("base/dir"), cwd);

  // Going up beyond root should normalize properly
  #[cfg(target_family = "unix")]
  {
    assert_eq_str!("../../../../file".absolutize_with("/a/b"), "/file");
  }

  #[cfg(target_family = "windows")]
  {
    assert_eq_str!("..\\..\\..\\..\\file".absolutize_with("C:\\a\\b"), "C:\\file");
  }
}
