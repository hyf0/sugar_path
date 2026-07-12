use std::borrow::Cow;
#[cfg(target_family = "unix")]
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
fn unix_absolutize_with_borrowed_and_owned_cwd_arguments() {
  assert_eq_str!("documents".absolutize_with("/home/user".as_path()), "/home/user/documents");

  let base_path = PathBuf::from("/var/log");
  assert_eq_str!("app.log".absolutize_with(base_path), "/var/log/app.log");
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
  assert_eq_str!("C:file".absolutize_with("C:\\base"), "C:\\base\\file");
  assert_eq_str!("C:C:foo".absolutize_with("C:\\base"), "C:\\base\\C:foo");
  assert_eq_str!("c:file".absolutize_with("C:\\base"), "c:\\base\\file");
  assert_eq_str!(r"C:file".absolutize_with(r"\\?\C:\base\D:dir"), r"\\?\C:\base\D:dir\file",);
  assert_eq_str!(r"C:file".absolutize_with(r"\\?\C:\base\foo/bar"), r"\\?\C:\base\foo/bar\file",);
  assert_eq_str!(r"C:file".absolutize_with(r"\\?\c:\base"), r"\\?\C:\base\file");
  assert_eq_str!("C:file".absolutize_with("D:\\base"), "C:file");
  assert_eq_str!("C:.\\file".absolutize_with("D:\\base"), "C:file");
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
fn absolutize_with_rejects_a_needed_nonabsolute_cwd() {
  let panic = std::panic::catch_unwind(|| "file".absolutize_with("relative/cwd"));
  assert!(panic.is_err());
}

#[test]
fn absolutize_with_does_not_validate_an_unused_cwd() {
  #[cfg(target_family = "unix")]
  assert_eq_str!("/already/absolute/".absolutize_with("relative/cwd"), "/already/absolute");
  #[cfg(target_family = "windows")]
  assert_eq_str!(r"C:\already\absolute\".absolutize_with("relative/cwd"), r"C:\already\absolute");
}

#[test]
fn absolutize_with_clamps_parents_at_an_absolute_root() {
  #[cfg(target_family = "unix")]
  {
    assert_eq_str!("../../../../file".absolutize_with("/a/b"), "/file");
  }

  #[cfg(target_family = "windows")]
  {
    assert_eq_str!("..\\..\\..\\..\\file".absolutize_with("C:\\a\\b"), "C:\\file");
  }
}

#[cfg(target_family = "unix")]
#[test]
fn unix_clean_absolute_paths_return_borrowed_cow() {
  for path in ["/", "/usr/local/bin", "/home/user/file.txt", "/foo/bar"] {
    assert!(
      matches!(p!(path).absolutize_with("/base"), Cow::Borrowed(_)),
      "expected borrowed Cow for clean absolute path {:?}",
      path,
    );
  }
}

#[cfg(target_family = "unix")]
#[test]
fn unix_dirty_or_relative_paths_return_owned_cow() {
  // Relative paths — always allocate
  for path in ["", "foo", "foo/bar", "./foo", "../foo"] {
    assert!(
      matches!(p!(path).absolutize_with("/base"), Cow::Owned(_)),
      "expected owned Cow for relative path {:?}",
      path,
    );
  }
  // Absolute but dirty — normalize allocates
  for path in ["/foo/../bar", "/foo//bar", "/foo/./bar", "/foo/bar/"] {
    assert!(
      matches!(p!(path).absolutize_with("/base"), Cow::Owned(_)),
      "expected owned Cow for dirty absolute path {:?}",
      path,
    );
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_clean_absolute_paths_return_borrowed_cow() {
  // On Windows, is_absolute() requires both a prefix (e.g. `C:`) and a root (`\`).
  // Root-relative paths like `\foo\bar` are NOT absolute.
  for path in [r"C:\foo\bar", r"C:\"] {
    assert!(
      matches!(p!(path).absolutize_with("C:\\base"), Cow::Borrowed(_)),
      "expected borrowed Cow for clean absolute path {:?}",
      path,
    );
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_dirty_or_relative_paths_return_owned_cow() {
  // Relative paths — always allocate
  for path in ["", "foo", r"foo\bar", r".\foo", r"..\foo"] {
    assert!(
      matches!(p!(path).absolutize_with("C:\\base"), Cow::Owned(_)),
      "expected owned Cow for relative path {:?}",
      path,
    );
  }
  // Root-relative (no prefix) — not absolute on Windows, always allocate
  for path in [r"\", r"\foo\bar"] {
    assert!(
      matches!(p!(path).absolutize_with("C:\\base"), Cow::Owned(_)),
      "expected owned Cow for root-relative path {:?}",
      path,
    );
  }
  // Drive-relative paths are converted to rooted paths.
  let drive_relative = "C:";
  assert!(
    matches!(p!(drive_relative).absolutize_with("C:\\base"), Cow::Owned(_)),
    "expected owned Cow for drive-relative path {:?}",
    drive_relative,
  );
  // Root-relative paths have no drive prefix.
  let root_relative = r"\foo\..\bar";
  assert!(
    matches!(p!(root_relative).absolutize_with("C:\\base"), Cow::Owned(_)),
    "expected owned Cow for root-relative path {:?}",
    root_relative,
  );
  // Absolute dirty paths are normalized into an owned result.
  for path in ["C:/foo", r"\\server\share\dir"] {
    assert!(
      matches!(p!(path).absolutize_with("C:\\base"), Cow::Owned(_)),
      "expected owned Cow for dirty absolute path {:?}",
      path,
    );
  }
}
