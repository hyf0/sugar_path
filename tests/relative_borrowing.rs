use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

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

fn assert_cow_variant(target: &Path, relative: Cow<'_, Path>, should_borrow: bool, label: &str) {
  assert_eq!(matches!(&relative, Cow::Borrowed(_)), should_borrow, "{label}: target {target:?}");
  if should_borrow {
    assert_borrows_from_target(target, &relative);
  }
}

#[test]
fn fallible_and_explicit_relative_preserve_output_and_cow_contracts() {
  #[cfg(target_family = "unix")]
  let cases = [
    ("/workspace/project/src/index.js", "/workspace/project", "src/index.js", true),
    ("/workspace/project", "/workspace/project", "", true),
    ("/workspace/project/src", "/workspace/project/dist", "../src", false),
    ("/workspace/project/./src", "/workspace/project", "src", false),
  ];
  #[cfg(target_family = "windows")]
  let cases = [
    (r"C:\workspace\project\src\index.js", r"C:\workspace\project", r"src\index.js", true),
    (r"C:\workspace\project", r"C:\workspace\project", "", true),
    (r"C:\workspace\project\src", r"C:\workspace\project\dist", r"..\src", false),
    (r"C:\workspace\project\.\src", r"C:\workspace\project", "src", false),
    (r"D:\workspace\project", r"C:\workspace\project", r"D:\workspace\project", false),
  ];

  for (target, base, expected, should_borrow) in cases {
    let target = Path::new(target);
    let fallible = target.try_relative(base).expect("absolute inputs do not need cwd");
    assert_eq!(fallible.as_os_str(), Path::new(expected).as_os_str());
    assert_cow_variant(target, fallible, should_borrow, "try_relative");

    let explicit = target.relative_with(base, Path::new("not/absolute"));
    assert_eq!(explicit.as_os_str(), Path::new(expected).as_os_str());
    assert_cow_variant(target, explicit, should_borrow, "relative_with");
  }
}

#[test]
fn owned_context_arguments_never_supply_relative_borrows() {
  #[cfg(target_family = "unix")]
  let (target, base, cwd, expected) =
    ("src/index.js", PathBuf::from("dist"), PathBuf::from("/workspace/project"), "../src/index.js");
  #[cfg(target_family = "windows")]
  let (target, base, cwd, expected) = (
    r"src\index.js",
    PathBuf::from("dist"),
    PathBuf::from(r"C:\workspace\project"),
    r"..\src\index.js",
  );

  fn relative_with_owned_context<'a>(
    target: &'a Path,
    base: PathBuf,
    cwd: PathBuf,
  ) -> Cow<'a, Path> {
    target.relative_with(base, cwd)
  }

  let target = Path::new(target);
  let relative = relative_with_owned_context(target, base, cwd);
  assert_eq!(relative.as_os_str(), Path::new(expected).as_os_str());
  assert_cow_variant(target, relative, false, "cwd-resolved relative_with");

  #[cfg(target_family = "unix")]
  let (target, base, expected) =
    ("/workspace/project/src/index.js", "/workspace/project", "src/index.js");
  #[cfg(target_family = "windows")]
  let (target, base, expected) =
    (r"C:\workspace\project\src\index.js", r"C:\workspace\project", r"src\index.js");

  let target = Path::new(target);
  let relative = relative_with_owned_context(
    target,
    PathBuf::from(base),
    PathBuf::from("unused/nonabsolute/cwd"),
  );
  assert_eq!(relative.as_os_str(), Path::new(expected).as_os_str());
  assert_cow_variant(target, relative, true, "clean relative_with with owned context");
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
