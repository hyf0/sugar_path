use std::{borrow::Cow, path::Path};

use sugar_path::SugarPath;

fn assert_borrows_from_target(target: &Path, relative: &Path) {
  let target_bytes = target.as_os_str().as_encoded_bytes();
  let relative_bytes = relative.as_os_str().as_encoded_bytes();
  let target_start = target_bytes.as_ptr() as usize;
  let target_end = target_start + target_bytes.len();
  let relative_start = relative_bytes.as_ptr() as usize;
  let relative_end = relative_start + relative_bytes.len();
  assert!(relative_start >= target_start && relative_end <= target_end);
}

#[cfg(target_family = "unix")]
#[test]
fn unix_descendants_borrow_the_target_suffix() {
  for (target, base, expected) in [
    (
      "/workspace/rolldown/crates/rolldown/src/module_loader/module_task.rs",
      "/workspace/rolldown",
      "crates/rolldown/src/module_loader/module_task.rs",
    ),
    (
      "/workspace/rolldown/crates/rolldown/src/module_loader/module_task.rs",
      "/workspace/rolldown/crates/rolldown/src/module_loader",
      "module_task.rs",
    ),
    ("/workspace/rolldown/src/", "/workspace/rolldown", "src"),
    ("/workspace/rolldown", "/workspace/rolldown", ""),
  ] {
    let target = Path::new(target);
    let Cow::Borrowed(relative) = target.relative(base) else {
      panic!("expected target {target:?} relative to {base:?} to borrow");
    };
    assert_eq!(relative.as_os_str().as_encoded_bytes(), expected.as_bytes());
    assert_borrows_from_target(target, relative);
  }
}

#[cfg(target_family = "unix")]
#[test]
fn unix_upward_dirty_and_invalid_results_remain_owned() {
  for (target, base) in [
    (
      "/workspace/rolldown/crates/rolldown/src/bundle/bundle.rs",
      "/workspace/rolldown/crates/rolldown/src/module_loader",
    ),
    ("/workspace/rolldown/src/./index.js", "/workspace/rolldown"),
  ] {
    let target = Path::new(target);
    let relative = target.relative(base);
    assert!(matches!(relative, Cow::Owned(_)), "target {target:?}, base {base:?}");
  }

  use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};
  let target = PathBuf::from(OsString::from_vec(b"/workspace/rolldown/invalid-\x80.js".to_vec()));
  let relative = target.relative("/workspace/rolldown");
  assert!(matches!(relative, Cow::Owned(_)));
}

#[cfg(target_family = "unix")]
#[test]
fn unix_descendants_with_redundant_internal_separators_are_owned() {
  let target = "/workspace/rolldown/src//index.js";
  let base = "/workspace/rolldown";
  let relative = Path::new(target).relative(base);
  assert!(matches!(relative, Cow::Owned(_)), "target {target:?}, base {base:?}");
  assert_eq!(relative.as_os_str().as_encoded_bytes(), b"src/index.js");
}

#[cfg(target_family = "unix")]
#[test]
fn string_values_borrow_through_the_str_impl() {
  let target = String::from("/workspace/rolldown/src/index.js");
  let Cow::Borrowed(relative) = target.relative("/workspace/rolldown") else {
    panic!("expected String target suffix to borrow");
  };
  assert_eq!(relative, Path::new("src/index.js"));
  assert_borrows_from_target(Path::new(&target), relative);
}

#[cfg(target_family = "unix")]
#[test]
fn borrowed_str_helpers_preserve_the_input_lifetime() {
  fn relative<'a>(target: &'a str, base: &Path) -> Cow<'a, Path> {
    target.relative(base)
  }

  let target = String::from("/workspace/rolldown/src/index.js");
  let relative = relative(&target, Path::new("/workspace/rolldown"));
  assert!(matches!(relative, Cow::Borrowed(_)));
  assert_eq!(relative, Path::new("src/index.js"));
}

#[cfg(target_family = "windows")]
#[test]
fn windows_native_descendants_borrow_the_target_suffix() {
  for (target, base, expected) in [
    (r"C:\workspace\rolldown\src\index.js", r"C:\workspace\rolldown", r"src\index.js"),
    (r"C:\workspace\rolldown/src\index.js", r"C:\workspace\rolldown", r"src\index.js"),
    (r"C:\workspace\rolldown\src\index.js", "c:/workspace/rolldown/src", "index.js"),
    (r"\\server\share\packages\app\index.js", r"\\server\share", r"packages\app\index.js"),
    (
      r"\\?\UNC\server\share\packages\app\index.js",
      r"\\?\UNC\server\share",
      r"packages\app\index.js",
    ),
    (r"\\?\C:\workspace\rolldown\src\index.js", r"\\?\C:\workspace\rolldown", r"src\index.js"),
    (r"C:\workspace\rolldown\src\", r"C:\workspace\rolldown", r"src"),
    (r"C:\workspace\rolldown", r"C:\workspace\rolldown", ""),
    ("C:/workspace/rolldown", "C:/workspace/rolldown", ""),
  ] {
    let target = Path::new(target);
    let Cow::Borrowed(relative) = target.relative(base) else {
      panic!("expected target {target:?} relative to {base:?} to borrow");
    };
    assert_eq!(relative.as_os_str().as_encoded_bytes(), expected.as_bytes());
    assert_borrows_from_target(target, relative);
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_noncanonical_descendants_are_canonical_and_owned() {
  let target = r"C:\workspace\rolldown\src\\index.js";
  let base = r"C:\workspace\rolldown";
  let relative = Path::new(target).relative(base);
  assert!(matches!(relative, Cow::Owned(_)), "target {target:?}, base {base:?}");
  assert_eq!(relative.as_os_str().as_encoded_bytes(), br"src\index.js");
}

#[cfg(target_family = "windows")]
#[test]
fn windows_owned_results_preserve_relative_semantics() {
  for (target, base) in [
    ("C:/workspace/rolldown/src/index.js", "C:/workspace/rolldown"),
    (r"C:\workspace\rolldown\src/index.js", r"C:\workspace\rolldown"),
    (r"C:\workspace\rolldown\src\index.js", r"C:\workspace\rolldown\dist"),
    (r"\\server\other\packages\app\index.js", r"\\server\share"),
    (r"dist\assets\index.js", r"dist\chunks"),
    (r"C:\workspace\rolldown\.\src\index.js", r"C:\workspace\rolldown"),
  ] {
    let target = Path::new(target);
    let relative = target.relative(base);
    assert!(matches!(relative, Cow::Owned(_)), "target {target:?}, base {base:?}");
  }
}
