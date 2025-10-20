use std::path::Path;
use sugar_path::SugarPath;

#[test]
fn test_as_path_on_str() {
  // Test that as_path() converts &str to Path correctly
  let string_path = "foo/bar/baz";
  let path = string_path.as_path();
  assert_eq!(path, Path::new("foo/bar/baz"));
}

#[test]
fn test_as_path_on_string() {
  // Test that as_path() works with String
  let string_path = String::from("hello/world");
  let path = string_path.as_path();
  assert_eq!(path, Path::new("hello/world"));
}

#[test]
fn test_as_path_with_absolute_paths() {
  // Test with absolute paths
  #[cfg(target_family = "unix")]
  {
    let abs_path = "/usr/local/bin";
    assert_eq!(abs_path.as_path(), Path::new("/usr/local/bin"));
  }

  #[cfg(target_family = "windows")]
  {
    let abs_path = "C:\\Windows\\System32";
    assert_eq!(abs_path.as_path(), Path::new("C:\\Windows\\System32"));
  }
}

#[test]
fn test_as_path_with_empty_string() {
  // Test with empty string
  let empty = "";
  assert_eq!(empty.as_path(), Path::new(""));
}

#[test]
fn test_as_path_with_dots() {
  // Test with relative path indicators
  assert_eq!(".".as_path(), Path::new("."));
  assert_eq!("..".as_path(), Path::new(".."));
  assert_eq!("./foo".as_path(), Path::new("./foo"));
  assert_eq!("../bar".as_path(), Path::new("../bar"));
}

#[test]
fn test_as_path_with_special_characters() {
  // Test with special characters in path
  assert_eq!("foo bar".as_path(), Path::new("foo bar"));
  assert_eq!("file.txt".as_path(), Path::new("file.txt"));
  assert_eq!("path-with-dash".as_path(), Path::new("path-with-dash"));
  assert_eq!("path_with_underscore".as_path(), Path::new("path_with_underscore"));
}

#[test]
fn test_as_path_chaining() {
  // Test that as_path() enables chaining with other SugarPath methods
  use std::path::PathBuf;

  let base = "foo/bar";
  let joined = base.as_path().join("baz");
  assert_eq!(joined, PathBuf::from("foo/bar/baz"));

  // Test chaining with normalize
  let normalized = "./foo/../bar".as_path().normalize();
  assert_eq!(normalized, PathBuf::from("bar"));
}

#[cfg(target_family = "unix")]
#[test]
fn test_as_path_unix_specific() {
  // Unix-specific path tests
  assert_eq!("/".as_path(), Path::new("/"));
  assert_eq!("/home/user/.config".as_path(), Path::new("/home/user/.config"));
  assert_eq!("~/documents".as_path(), Path::new("~/documents"));
}

#[cfg(target_family = "windows")]
#[test]
fn test_as_path_windows_specific() {
  // Windows-specific path tests
  assert_eq!("C:".as_path(), Path::new("C:"));
  assert_eq!("C:\\".as_path(), Path::new("C:\\"));
  assert_eq!("\\\\server\\share".as_path(), Path::new("\\\\server\\share"));
  assert_eq!("file:stream".as_path(), Path::new("file:stream"));
}
