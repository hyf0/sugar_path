#[cfg(not(windows))]
use std::path::PathBuf;
use sugar_path::SugarPath;
mod test_utils;

#[cfg(target_family = "unix")]
#[test]
fn unix_absolutize_with() {
  // Basic absolutize_with tests
  assert_eq!("./world".absolutize_with("/hello"), pb!("/hello/world"));
  assert_eq!("../world".absolutize_with("/hello"), pb!("/world"));
  assert_eq!("world".absolutize_with("/hello"), pb!("/hello/world"));

  // With absolute paths as input
  assert_eq!("/absolute".absolutize_with("/base"), pb!("/absolute"));
  assert_eq!("/usr/bin".absolutize_with("/home"), pb!("/usr/bin"));

  // With dots in paths
  assert_eq!("./a/./b/../c".absolutize_with("/base"), pb!("/base/a/c"));
  assert_eq!("../a/../b".absolutize_with("/base/dir"), pb!("/base/b"));

  // Empty path
  assert_eq!("".absolutize_with("/base"), pb!("/base"));
  assert_eq!(".".absolutize_with("/base"), pb!("/base"));

  // Multiple levels up
  assert_eq!("../../file".absolutize_with("/a/b/c"), pb!("/a/file"));
  assert_eq!("../../../file".absolutize_with("/a/b/c"), pb!("/file"));

  // Complex paths
  assert_eq!("./foo/../bar/./baz".absolutize_with("/root"), pb!("/root/bar/baz"));
  assert_eq!("a/b/../../c".absolutize_with("/base"), pb!("/base/c"));
}

#[cfg(target_family = "unix")]
#[test]
fn unix_absolutize_with_trailing_slash() {
  // Test with trailing slashes
  assert_eq!("world/".absolutize_with("/hello/"), pb!("/hello/world/"));
  assert_eq!("./world/".absolutize_with("/hello"), pb!("/hello/world/"));
}

#[cfg(target_family = "unix")]
#[test]
fn unix_absolutize_with_string_types() {
  // Test with different string types as base
  let base = String::from("/home/user");
  assert_eq!("documents".absolutize_with(&base), pb!("/home/user/documents"));
  assert_eq!("../downloads".absolutize_with(base.as_str()), pb!("/home/downloads"));

  // Test with PathBuf as base
  let base_path = PathBuf::from("/var/log");
  assert_eq!("app.log".absolutize_with(&base_path), pb!("/var/log/app.log"));
}

#[cfg(target_family = "windows")]
#[test]
fn windows_absolutize_with() {
  // Basic absolutize_with tests
  assert_eq!(".\\world".absolutize_with("C:\\hello"), pb!("C:\\hello\\world"));
  assert_eq!("..\\world".absolutize_with("C:\\hello"), pb!("C:\\world"));
  assert_eq!("world".absolutize_with("C:\\hello"), pb!("C:\\hello\\world"));

  // With absolute paths as input
  assert_eq!("D:\\absolute".absolutize_with("C:\\base"), pb!("D:\\absolute"));
  assert_eq!("C:\\Windows".absolutize_with("C:\\Users"), pb!("C:\\Windows"));

  // With dots in paths
  assert_eq!(".\\a\\.\\b\\..\\c".absolutize_with("C:\\base"), pb!("C:\\base\\a\\c"));
  assert_eq!("..\\a\\..\\b".absolutize_with("C:\\base\\dir"), pb!("C:\\base\\b"));

  // Empty path
  assert_eq!("".absolutize_with("C:\\base"), pb!("C:\\base"));
  assert_eq!(".".absolutize_with("C:\\base"), pb!("C:\\base"));

  // Multiple levels up
  assert_eq!("..\\..\\file".absolutize_with("C:\\a\\b\\c"), pb!("C:\\a\\file"));

  // Drive-relative paths
  assert_eq!("C:file".absolutize_with("D:\\base"), pb!("C:\\file"));
  assert_eq!("C:.\\file".absolutize_with("D:\\base"), pb!("C:\\file"));
}

#[cfg(target_family = "windows")]
#[test]
fn windows_absolutize_with_unc_paths() {
  // UNC path tests
  assert_eq!("file".absolutize_with("\\\\server\\share"), pb!("\\\\server\\share\\file"));
  assert_eq!(
    "..\\other".absolutize_with("\\\\server\\share\\folder"),
    pb!("\\\\server\\share\\other")
  );
  assert_eq!("\\\\other\\share".absolutize_with("\\\\server\\share"), pb!("\\\\other\\share"));
}

#[cfg(target_family = "windows")]
#[test]
fn windows_absolutize_with_mixed_separators() {
  // Test with mixed separators
  assert_eq!("sub/folder".absolutize_with("C:\\base"), pb!("C:\\base\\sub\\folder"));
  assert_eq!("./sub\\folder".absolutize_with("C:/base"), pb!("C:\\base\\sub\\folder"));
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
    assert_eq!("../../../../file".absolutize_with("/a/b"), pb!("/file"));
  }

  #[cfg(target_family = "windows")]
  {
    assert_eq!("..\\..\\..\\..\\file".absolutize_with("C:\\a\\b"), pb!("C:\\file"));
  }
}
