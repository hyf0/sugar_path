#![cfg(any(target_family = "unix", target_family = "windows"))]

use std::{borrow::Cow, path::Path};

use sugar_path::SugarPath;

#[cfg(target_family = "unix")]
mod unix {
  use std::{
    env,
    ffi::OsString,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
  };

  use super::*;

  fn path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(OsString::from_vec(bytes.to_vec()))
  }

  fn assert_bytes(actual: &Path, expected: &Path, context: &str) {
    assert_eq!(actual.as_os_str().as_bytes(), expected.as_os_str().as_bytes(), "{context}",);
  }

  fn assert_ambient(input: &Path, expected: &Path, require_owned: bool, name: &str) {
    let strict = input.absolutize();
    assert_bytes(&strict, expected, &format!("{name} strict"));
    if require_owned {
      assert!(matches!(strict, Cow::Owned(_)), "{name} strict should own");
    }

    let fallible = input.try_absolutize().expect("fixture should resolve against cwd");
    assert_bytes(&fallible, expected, &format!("{name} try"));
    if require_owned {
      assert!(matches!(fallible, Cow::Owned(_)), "{name} try should own");
    }
  }

  #[test]
  fn ambient_absolutization_preserves_invalid_unix_bytes() {
    let clean = path(b"/sugar-path/invalid-\x80/file");
    assert_ambient(&clean, &clean, false, "clean absolute");

    let dirty = path(b"/sugar-path/invalid-\x80/./file/");
    let normalized = path(b"/sugar-path/invalid-\x80/file");
    assert_ambient(&dirty, &normalized, true, "dirty absolute");

    let relative = path(b"pkg/invalid-\x80/./file/");
    let expected =
      env::current_dir().expect("read current directory").join(path(b"pkg/invalid-\x80/file"));
    assert_ambient(&relative, &expected, true, "relative");
  }
}

#[cfg(target_family = "windows")]
mod windows {
  use std::{
    env,
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
  };

  use super::*;

  const INVALID_UNIT: u16 = 0xd800;

  fn invalid_path(before: &str, after: &str) -> PathBuf {
    let mut wide: Vec<u16> = before.encode_utf16().collect();
    wide.push(INVALID_UNIT);
    wide.extend(after.encode_utf16());
    PathBuf::from(OsString::from_wide(&wide))
  }

  fn assert_wide(actual: &Path, expected: &Path, context: &str) {
    assert_eq!(
      actual.as_os_str().encode_wide().collect::<Vec<_>>(),
      expected.as_os_str().encode_wide().collect::<Vec<_>>(),
      "{context}",
    );
  }

  fn current_ordinary_disk_drive(path: &Path) -> Option<u8> {
    use std::path::{Component, Prefix};

    match path.components().next() {
      Some(Component::Prefix(prefix)) => match prefix.kind() {
        Prefix::Disk(drive) => Some(drive),
        _ => None,
      },
      _ => None,
    }
  }

  fn assert_ambient(input: &Path, expected: &Path, require_owned: bool, name: &str) {
    let strict = input.absolutize();
    assert_wide(&strict, expected, &format!("{name} strict"));
    if require_owned {
      assert!(matches!(strict, Cow::Owned(_)), "{name} strict should own");
    }

    let fallible = input.try_absolutize().expect("fixture should resolve against cwd");
    assert_wide(&fallible, expected, &format!("{name} try"));
    if require_owned {
      assert!(matches!(fallible, Cow::Owned(_)), "{name} try should own");
    }
  }

  #[test]
  fn ambient_absolutization_preserves_invalid_windows_units() {
    let cwd = env::current_dir().expect("read current directory");

    let clean = invalid_path(r"C:\sugar-path\invalid-", r"\file");
    assert_ambient(&clean, &clean, false, "clean absolute");

    let dirty = invalid_path(r"C:\sugar-path\invalid-", r"\.\file\");
    let normalized = invalid_path(r"C:\sugar-path\invalid-", r"\file");
    assert_ambient(&dirty, &normalized, true, "dirty absolute");

    let relative = invalid_path(r"pkg\invalid-", r"\.\file\");
    let expected = cwd.join(invalid_path(r"pkg\invalid-", r"\file"));
    assert_ambient(&relative, &expected, true, "relative");

    let root_relative = invalid_path(r"\pkg\invalid-", r"\.\file\");
    let mut expected = cwd.clone();
    expected.push(invalid_path(r"\pkg\invalid-", r"\file"));
    assert_ambient(&root_relative, &expected, true, "root relative");

    if let Some(drive) = current_ordinary_disk_drive(&cwd) {
      let prefix = format!("{}:pkg\\invalid-", drive as char);
      let drive_relative = invalid_path(&prefix, r"\.\file\");
      let expected = cwd.join(invalid_path(r"pkg\invalid-", r"\file"));
      assert_ambient(&drive_relative, &expected, false, "drive relative");
    }
  }
}
