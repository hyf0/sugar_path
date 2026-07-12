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
  // Dual algorithm: dirty into_normalized uses stack-arena reuse; borrowed
  // normalize uses normalize_inner. Keep a broad corpus so the two stay aligned.
  #[cfg(target_family = "unix")]
  let cases = [
    "foo/./bar/../baz",
    "../../foo/../bar",
    "foo//bar/",
    "",
    ".",
    "./",
    "foo/..",
    "foo/../",
    "a/b/../../c/./d//e/",
    "../../../x",
    "/a/./b/../c//",
    "/../file",
    "name/../name/../name/../out",
  ];
  #[cfg(target_family = "windows")]
  let cases = [
    r"foo\.\bar\..\baz",
    r"..\..\foo\..\bar",
    r"foo\\bar\",
    "",
    ".",
    r".\",
    r"foo\..",
    r"foo\..\",
    r"a\b\..\..\c\.\d\\e\",
    r"..\..\..\x",
    r"C:\a\.\b\..\c\\",
    r"C:\..\file",
    r"dir\..\C:foo",
    r"dir\..\C:foo\..",
    r"dir\..\C:foo\..\",
    r"C:foo\.\bar",
    r"\\?\c:\foo\..",
    r"\\.\PIPE\foo\..",
    r"\\?\UNC\server\share\foo\..",
    r"\\?\Volume{abc}\foo\..",
    r"name\..\name\..\name\..\out",
  ];

  for input in cases {
    let path = PathBuf::from(input);
    let expected = path.normalize().into_owned();
    assert_eq!(path.clone().into_normalized().as_os_str(), expected.as_os_str(), "input {input:?}");
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_owned_normalization_keeps_prefix_only_and_collapsed_relative_spellings() {
  let cases = [
    (r"dir\..\C:foo\..", "."),
    (r"dir\..\C:foo\..\", r".\"),
    (r"c:foo\..", "c:."),
    (r"c:foo\..\", r"c:.\"),
  ];

  for (input, expected) in cases {
    let path = PathBuf::from(input);
    assert_eq!(path.normalize().as_os_str(), Path::new(expected).as_os_str(), "input {input:?}");
    assert_eq!(
      path.into_normalized().as_os_str(),
      Path::new(expected).as_os_str(),
      "input {input:?}",
    );
  }
}

fn dirty_path_with_depth(depth: usize) -> PathBuf {
  let mut path = PathBuf::with_capacity(256);
  for _ in 0..depth {
    path.push("a");
  }
  path.push(".");
  path
}

#[test]
fn into_normalized_reuses_the_component_and_byte_arena_boundaries() {
  let path = dirty_path_with_depth(24);
  let identity = buffer_identity(&path);
  let expected = path.normalize().into_owned();
  let normalized = path.into_normalized();
  assert_eq!(normalized.as_os_str(), expected.as_os_str());
  assert_eq!(buffer_identity(&normalized), identity);

  let path = dirty_path_with_depth(25);
  let expected = path.normalize().into_owned();
  assert_eq!(path.into_normalized().as_os_str(), expected.as_os_str());

  let separator = std::path::MAIN_SEPARATOR;
  let input = format!(".{separator}{}", "x".repeat(510));
  assert_eq!(input.len(), 512);
  let path = PathBuf::from(input);
  let identity = buffer_identity(&path);
  let expected = path.normalize().into_owned();
  let normalized = path.into_normalized();
  assert_eq!(normalized.as_os_str(), expected.as_os_str());
  assert_eq!(buffer_identity(&normalized), identity);

  let input = format!(".{separator}{}", "x".repeat(511));
  assert_eq!(input.len(), 513);
  let path = PathBuf::from(input);
  let expected = path.normalize().into_owned();
  assert_eq!(path.into_normalized().as_os_str(), expected.as_os_str());
}

#[cfg(target_family = "windows")]
#[test]
fn into_normalized_reuses_long_unc_prefixes_without_a_temporary_heap_buffer() {
  let server = "s".repeat(57);
  let prefix = format!(r"\\{server}\share");
  assert_eq!(prefix.len(), 65);
  let mut path = PathBuf::with_capacity(256);
  path.push(&prefix);
  for _ in 0..24 {
    path.push("a");
  }
  path.push(".");
  assert_eq!(path.as_os_str().len(), 115);
  let identity = buffer_identity(&path);
  let expected = path.normalize().into_owned();
  let normalized = path.into_normalized();
  assert_eq!(normalized.as_os_str(), expected.as_os_str());
  assert_eq!(buffer_identity(&normalized), identity);

  let mut deep = PathBuf::from(prefix);
  for _ in 0..25 {
    deep.push("a");
  }
  deep.push(".");
  assert_eq!(deep.as_os_str().len(), 117);
  let expected = deep.normalize().into_owned();
  assert_eq!(deep.into_normalized().as_os_str(), expected.as_os_str());
}

#[test]
fn into_normalized_matches_borrowed_api_on_overflow_fallbacks() {
  // Depth > 32 reaches the allocation-free deep replay in the non-Unix
  // normalizer after the owned 24-component stack has already overflowed.
  let mut deep = PathBuf::new();
  for i in 0..34 {
    deep.push(format!("level-{i}"));
  }
  deep.push("..");
  deep.push("..");
  deep.push("tail");
  let mut expected = PathBuf::new();
  for i in 0..32 {
    expected.push(format!("level-{i}"));
  }
  expected.push("tail");
  assert_eq!(deep.normalize().as_os_str(), expected.as_os_str());
  assert_eq!(deep.into_normalized().as_os_str(), expected.as_os_str());

  // Encoded length > 512 forces the long-path → normalize_inner path.
  let long_component = "x".repeat(280);
  #[cfg(target_family = "unix")]
  let long = format!("a/{long_component}/./b/{long_component}/../c/");
  #[cfg(target_family = "windows")]
  let long = format!(r"a\{long_component}\.\b\{long_component}\..\c\");
  assert!(long.len() > 512, "fixture must exceed the owned-normalize stack arena");
  let path = PathBuf::from(&long);
  let expected = path.normalize().into_owned();
  assert_eq!(path.clone().into_normalized().as_os_str(), expected.as_os_str());
}

#[test]
fn owned_slash_apis_reuse_valid_unicode_buffers() {
  #[cfg(target_family = "unix")]
  let cases = [
    (
      "/workspace/rolldown/crates/rolldown/src/module_loader/module_task.rs",
      "/workspace/rolldown/crates/rolldown/src/module_loader/module_task.rs",
    ),
    (r"/root/./β/../tail//literal\name//", r"/root/./β/../tail//literal\name//"),
  ];
  #[cfg(target_family = "windows")]
  let cases = [
    (
      r"C:\workspace\rolldown\crates\rolldown\src\module_loader\module_task.rs",
      "C:/workspace/rolldown/crates/rolldown/src/module_loader/module_task.rs",
    ),
    (r"\\server\share\.\β\..\tail\\foreign/name\\", "//server/share/./β/../tail//foreign/name//"),
    ("//server/share/./β/../tail//foreign/name//", "//server/share/./β/../tail//foreign/name//"),
  ];

  for (input, expected) in cases {
    let path = owned_path_with_capacity(input);
    let identity = buffer_identity(&path);
    let slash = path.into_slash();
    assert_eq!(slash, expected, "strict: input {input:?}");
    assert_eq!((slash.as_ptr(), slash.capacity()), identity, "strict: input {input:?}");

    let path = owned_path_with_capacity(input);
    let identity = buffer_identity(&path);
    let slash = path.try_into_slash().expect("the fixture is valid Unicode");
    assert_eq!(slash, expected, "try: input {input:?}");
    assert_eq!((slash.as_ptr(), slash.capacity()), identity, "try: input {input:?}");

    let path = owned_path_with_capacity(input);
    let identity = buffer_identity(&path);
    let slash = path.into_slash_lossy();
    assert_eq!(slash, expected, "lossy: input {input:?}");
    assert_eq!((slash.as_ptr(), slash.capacity()), identity, "lossy: input {input:?}");
  }
}

#[cfg(target_family = "unix")]
#[test]
fn owned_apis_preserve_and_replace_invalid_unix_encoding() {
  use std::{
    ffi::OsString,
    os::unix::ffi::{OsStrExt, OsStringExt},
  };

  let normalize_input = PathBuf::from(OsString::from_vec(b"dir/invalid-\x80/./file".to_vec()));
  let normalized = normalize_input.into_normalized();
  assert_eq!(normalized.as_os_str().as_bytes(), b"dir/invalid-\x80/file");

  let input = PathBuf::from(OsString::from_vec(b"./dir//invalid-\x80/../tail/".to_vec()));

  let strict = input.clone();
  assert!(std::panic::catch_unwind(move || strict.into_slash()).is_err());

  let recoverable = input.clone();
  let returned =
    recoverable.try_into_slash().expect_err("invalid Unix encoding must be returned unchanged");
  assert_eq!(returned.as_os_str().as_bytes(), input.as_os_str().as_bytes());
  assert_eq!(input.into_slash_lossy(), "./dir//invalid-\u{fffd}/../tail/");
}

#[cfg(target_family = "windows")]
#[test]
fn owned_apis_preserve_and_replace_invalid_windows_encoding() {
  use std::{
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
  };

  let mut normalize_input_wide: Vec<u16> = r"C:\workspace\invalid-".encode_utf16().collect();
  normalize_input_wide.push(0xd800);
  normalize_input_wide.extend(r"\.\file".encode_utf16());
  let normalize_input = PathBuf::from(OsString::from_wide(&normalize_input_wide));

  let mut normalized_wide: Vec<u16> = r"C:\workspace\invalid-".encode_utf16().collect();
  normalized_wide.push(0xd800);
  normalized_wide.extend(r"\file".encode_utf16());
  assert_eq!(
    normalize_input.into_normalized().as_os_str().encode_wide().collect::<Vec<_>>(),
    normalized_wide,
  );

  let mut input_wide: Vec<u16> = r".\dir\\invalid-".encode_utf16().collect();
  input_wide.push(0xd800);
  input_wide.extend(r"\..\tail\".encode_utf16());
  let input = PathBuf::from(OsString::from_wide(&input_wide));

  let strict = input.clone();
  assert!(std::panic::catch_unwind(move || strict.into_slash()).is_err());

  let recoverable = input.clone();
  let returned =
    recoverable.try_into_slash().expect_err("invalid Windows encoding must be returned unchanged");
  assert_eq!(returned.as_os_str().encode_wide().collect::<Vec<_>>(), input_wide);
  assert_eq!(input.into_slash_lossy(), "./dir//invalid-\u{fffd}/../tail/");

  let mut no_native_separator_wide: Vec<u16> = "invalid-".encode_utf16().collect();
  no_native_separator_wide.push(0xd800);
  no_native_separator_wide.extend("/tail".encode_utf16());
  let no_native_separator = PathBuf::from(OsString::from_wide(&no_native_separator_wide));
  assert_eq!(no_native_separator.into_slash_lossy(), "invalid-\u{fffd}/tail");
}
