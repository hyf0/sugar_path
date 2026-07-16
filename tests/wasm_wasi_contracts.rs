#![cfg(all(target_os = "wasi", target_env = "p1"))]

use std::{
  ffi::OsString,
  os::wasi::ffi::{OsStrExt, OsStringExt},
  path::{Path, PathBuf},
};

use sugar_path::{SugarPath, SugarPathBuf};

fn assert_normalizes_exactly(input: &Path, expected: &[u8]) {
  let normalized = input.normalize();
  assert_eq!(normalized.as_os_str().as_bytes(), expected);
  assert_eq!(normalized.normalize().as_os_str().as_bytes(), expected);

  let consumed = input.to_path_buf().into_normalized();
  assert_eq!(consumed.as_os_str().as_bytes(), expected);
  assert_eq!(consumed.into_normalized().as_os_str().as_bytes(), expected);
}

#[test]
fn normalize_preserves_wasi_spelling_and_native_encoding() {
  let dot = Path::new(".");
  let dot_slash = Path::new("./");
  assert_eq!(dot, dot_slash);
  assert_ne!(dot.as_os_str().as_bytes(), dot_slash.as_os_str().as_bytes());
  assert_normalizes_exactly(dot, b".");
  assert_normalizes_exactly(dot_slash, b"./");

  assert_normalizes_exactly(Path::new("workspace/src/lib.rs"), b"workspace/src/lib.rs");
  assert_normalizes_exactly(
    Path::new("workspace/./src/../dist/assets/"),
    b"workspace/dist/assets/",
  );

  let invalid = PathBuf::from(OsString::from_vec(b"workspace/invalid-\x80/./file".to_vec()));
  assert_normalizes_exactly(&invalid, b"workspace/invalid-\x80/file");
}

#[test]
fn absolutize_uses_explicit_context_without_host_filesystem_access() {
  assert_eq!(
    Path::new("/workspace/src/lib.rs").absolutize_with("unused/relative/cwd"),
    Path::new("/workspace/src/lib.rs"),
  );
  assert_eq!(
    Path::new("src/./loader/../lib.rs").absolutize_with("/workspace"),
    Path::new("/workspace/src/lib.rs"),
  );
}

#[test]
fn relative_apis_cover_ambient_independent_and_explicit_contexts() {
  assert_eq!(Path::new("/workspace/src/lib.rs").relative("/workspace"), Path::new("src/lib.rs"),);
  assert_eq!(
    Path::new("/workspace/src").try_relative("/workspace/dist").unwrap(),
    Path::new("../src"),
  );
  assert_eq!(
    Path::new("src/lib.rs").relative_with("dist/chunk.js", "/workspace"),
    Path::new("../../src/lib.rs"),
  );
  assert_eq!(Path::new("../../dist/assets").relative("../../dist/assets"), Path::new(""));
}

#[test]
fn slash_policies_cover_valid_and_invalid_wasi_encoding() {
  let valid = Path::new("workspace/src/lib.rs");
  assert_eq!(valid.to_slash(), "workspace/src/lib.rs");
  assert_eq!(valid.try_to_slash().as_deref(), Some("workspace/src/lib.rs"));
  assert_eq!(valid.to_slash_lossy(), "workspace/src/lib.rs");

  let invalid = PathBuf::from(OsString::from_vec(b"workspace/invalid-\x80/file".to_vec()));
  assert!(invalid.try_to_slash().is_none());
  assert_eq!(invalid.to_slash_lossy(), "workspace/invalid-\u{fffd}/file");

  let returned = invalid.clone().try_into_slash().unwrap_err();
  assert_eq!(returned.as_os_str().as_bytes(), invalid.as_os_str().as_bytes());
  assert_eq!(invalid.into_slash_lossy(), "workspace/invalid-\u{fffd}/file");
}

#[test]
fn consuming_and_string_apis_preserve_wasi_results() {
  assert_eq!(
    PathBuf::from("workspace/./src/../dist/lib.rs").into_normalized().as_os_str().as_bytes(),
    b"workspace/dist/lib.rs",
  );
  assert_eq!(PathBuf::from("workspace/src/lib.rs").into_slash(), "workspace/src/lib.rs");

  let owned = String::from("workspace/src/lib.rs");
  assert_eq!(owned.as_path(), Path::new("workspace/src/lib.rs"));
  assert_eq!(owned.normalize().as_os_str().as_bytes(), b"workspace/src/lib.rs");
  assert_eq!(owned.to_slash(), "workspace/src/lib.rs");
}
