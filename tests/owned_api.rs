use std::path::{Path, PathBuf};

use sugar_path::{SugarPath, SugarPathBuf};

fn buffer_identity(path: &PathBuf) -> (*const u8, usize) {
  (path.as_os_str().as_encoded_bytes().as_ptr(), path.capacity())
}

fn owned_path_with_capacity(path: &str) -> PathBuf {
  let mut owned = PathBuf::with_capacity(256);
  owned.push(path);
  owned
}

#[test]
fn into_normalized_reuses_clean_owned_buffers() {
  #[cfg(target_family = "unix")]
  let cases = [
    "/workspace/rolldown/crates/rolldown/src/module_loader/module_task.rs",
    "../../chunks/shared.js",
  ];
  #[cfg(target_family = "windows")]
  let cases = [
    r"C:\workspace\rolldown\crates\rolldown\src\module_loader\module_task.rs",
    r"..\..\chunks\shared.js",
  ];

  for input in cases {
    let path = owned_path_with_capacity(input);
    let identity = buffer_identity(&path);
    let normalized = path.into_normalized();

    assert_eq!(normalized.as_os_str(), Path::new(input).as_os_str());
    assert_eq!(buffer_identity(&normalized), identity);
  }
}

#[test]
fn into_normalized_reuses_current_directory_buffers_without_confusing_borrowed_dot() {
  #[cfg(target_family = "unix")]
  let cases = [("", "."), (".", "."), ("./", "./"), ("foo/..", ".")];
  #[cfg(target_family = "windows")]
  let cases = [("", "."), (".", "."), (r".\", r".\"), (r"foo\..", ".")];

  for (input, expected) in cases {
    let path = owned_path_with_capacity(input);
    let identity = buffer_identity(&path);
    let normalized = path.into_normalized();

    assert_eq!(normalized.as_os_str(), Path::new(expected).as_os_str(), "input {input:?}");
    assert_eq!(buffer_identity(&normalized), identity, "input {input:?}");
  }
}

#[test]
fn into_normalized_matches_borrowed_api_for_dirty_paths() {
  #[cfg(target_family = "unix")]
  let cases = ["foo/./bar/../baz", "../../foo/../bar", "foo//bar/"];
  #[cfg(target_family = "windows")]
  let cases = [r"foo\.\bar\..\baz", r"..\..\foo\..\bar", r"foo\\bar\"];

  for input in cases {
    let path = owned_path_with_capacity(input);
    let expected = path.normalize().into_owned();
    assert_eq!(path.into_normalized().as_os_str(), expected.as_os_str(), "input {input:?}");
  }
}

#[test]
fn owned_slash_apis_reuse_valid_unicode_buffers() {
  #[cfg(target_family = "unix")]
  let (input, expected) = (
    "/workspace/rolldown/crates/rolldown/src/module_loader/module_task.rs",
    "/workspace/rolldown/crates/rolldown/src/module_loader/module_task.rs",
  );
  #[cfg(target_family = "windows")]
  let (input, expected) = (
    r"C:\workspace\rolldown\crates\rolldown\src\module_loader\module_task.rs",
    "C:/workspace/rolldown/crates/rolldown/src/module_loader/module_task.rs",
  );

  let path = owned_path_with_capacity(input);
  let identity = buffer_identity(&path);
  let slash = path.into_slash();

  assert_eq!(slash, expected);
  assert_eq!((slash.as_ptr(), slash.capacity()), identity);

  let path = owned_path_with_capacity(input);
  let identity = buffer_identity(&path);
  let slash = path.try_into_slash().expect("the fixture is valid Unicode");
  assert_eq!(slash, expected);
  assert_eq!((slash.as_ptr(), slash.capacity()), identity);
}

#[cfg(target_family = "unix")]
#[test]
fn owned_apis_preserve_and_replace_invalid_unix_encoding() {
  use std::{
    ffi::OsString,
    os::unix::ffi::{OsStrExt, OsStringExt},
  };

  let input = PathBuf::from(OsString::from_vec(b"dir/invalid-\x80/./file".to_vec()));
  let normalized = input.clone().into_normalized();
  assert_eq!(normalized.as_os_str().as_bytes(), b"dir/invalid-\x80/file");
  let returned =
    input.clone().try_into_slash().expect_err("invalid Unix encoding must be returned unchanged");
  assert_eq!(returned.as_os_str().as_bytes(), input.as_os_str().as_bytes());
  assert_eq!(input.into_slash_lossy(), "dir/invalid-\u{fffd}/./file");
}

#[cfg(target_family = "windows")]
#[test]
fn owned_apis_preserve_and_replace_invalid_windows_encoding() {
  use std::{
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
  };

  let mut input_wide: Vec<u16> = r"C:\workspace\invalid-".encode_utf16().collect();
  input_wide.push(0xd800);
  input_wide.extend(r"\.\file".encode_utf16());
  let input = PathBuf::from(OsString::from_wide(&input_wide));

  let mut expected_wide: Vec<u16> = r"C:\workspace\invalid-".encode_utf16().collect();
  expected_wide.push(0xd800);
  expected_wide.extend(r"\file".encode_utf16());
  assert_eq!(
    input.clone().into_normalized().as_os_str().encode_wide().collect::<Vec<_>>(),
    expected_wide,
  );
  let returned = input
    .clone()
    .try_into_slash()
    .expect_err("invalid Windows encoding must be returned unchanged");
  assert_eq!(returned.as_os_str().encode_wide().collect::<Vec<_>>(), input_wide);
  assert_eq!(input.into_slash_lossy(), "C:/workspace/invalid-\u{fffd}/./file");
}
