#![cfg(any(unix, windows))]

use std::path::{Path, PathBuf};

use sugar_path::SugarPath;

fn oracle_with_cwd(target: &Path, base: &Path, cwd: &Path) -> PathBuf {
  target.relative_with(base, cwd).into_owned()
}

fn assert_exact(base: &str, target: &str, expected: &str) {
  let actual = Path::new(target).relative(Path::new(base));
  assert_eq!(
    actual.as_os_str(),
    Path::new(expected).as_os_str(),
    "target {target:?}, base {base:?}",
  );
}

#[cfg(unix)]
#[test]
fn unix_lexical_relative_matrix_has_exact_stable_results() {
  for (base, target, expected) in [
    ("dist/chunks", "dist/assets/index.js", "../assets/index.js"),
    ("dist/./chunks/../chunks", "./dist/assets/./temp/../index.js", "../assets/index.js"),
    ("../../dist/chunks", "../../dist/assets/index.js", "../assets/index.js"),
    ("../a/../../dist/chunks", "a/../../../dist/assets/index.js", "../assets/index.js"),
    (".", "", ""),
    ("./", "foo/..", ""),
    ("dist/chunks", ".", "../.."),
    ("", "dist/assets", "dist/assets"),
    ("dist//chunks/", "dist/assets/", "../assets"),
  ] {
    assert_exact(base, target, expected);
  }
}

#[cfg(windows)]
#[test]
fn windows_lexical_relative_matrix_has_exact_stable_results() {
  for (base, target, expected) in [
    (r"dist\chunks", r"dist\assets\index.js", r"..\assets\index.js"),
    (r"dist\.\chunks\..\chunks", r".\dist\assets\.\temp\..\index.js", r"..\assets\index.js"),
    (r"..\..\dist\chunks", r"..\..\dist\assets\index.js", r"..\assets\index.js"),
    (r"..\a\..\..\dist\chunks", r"a\..\..\..\dist\assets\index.js", r"..\assets\index.js"),
    (".", "", ""),
    (r".\", r"foo\..", ""),
    (r"DIST\Chunks", r"dist\assets", r"..\assets"),
    (r"dist\chunks", ".", r"..\.."),
    ("", r"dist\assets", r"dist\assets"),
  ] {
    assert_exact(base, target, expected);
  }
}

#[cfg(unix)]
const SAFE_GROUPS: &[&[&str]] = &[
  &["", ".", "./", "foo/..", "dist/assets", "./dist/./assets/"],
  &["..", "../.", "../dist", "a/../../dist", "../dist/assets/.."],
  &["../..", "../../.", "../../dist", "a/../../../dist", "../../dist/assets/.."],
];

#[cfg(windows)]
const SAFE_GROUPS: &[&[&str]] = &[
  &["", ".", r".\", r"foo\..", r"dist\assets", r".\dist\.\assets\"],
  &["..", r"..\.", r"..\dist", r"a\..\..\dist", r"..\dist\assets\.."],
  &[r"..\..", r"..\..\.", r"..\..\dist", r"a\..\..\..\dist", r"..\..\dist\assets\.."],
];

#[cfg(unix)]
const CWDS: &[&str] = &["/", "/one", "/one/two", "/one/two/three"];

#[cfg(windows)]
const CWDS: &[&str] = &[r"C:\", r"C:\one", r"C:\one\two", r"C:\one\two\three"];

#[test]
fn equal_unresolved_parent_counts_are_cwd_independent() {
  for group in SAFE_GROUPS {
    for target in *group {
      for base in *group {
        let actual = Path::new(target).relative(Path::new(base));
        for cwd in CWDS {
          let expected = oracle_with_cwd(Path::new(target), Path::new(base), Path::new(cwd));
          assert_eq!(
            actual.as_os_str(),
            expected.as_os_str(),
            "target {target:?}, base {base:?}, cwd {cwd:?}",
          );
        }
      }
    }
  }
}

#[test]
fn unequal_unresolved_parent_counts_use_the_cwd_dependent_fallback() {
  let cwd = std::env::current_dir().expect("read cwd");
  #[cfg(unix)]
  let cases = [
    ("../../dist/assets", "../dist/chunks"),
    ("a/../../target", "base"),
    ("target", "a/../../base"),
  ];
  #[cfg(windows)]
  let cases = [
    (r"..\..\dist\assets", r"..\dist\chunks"),
    (r"a\..\..\target", "base"),
    ("target", r"a\..\..\base"),
  ];

  for (target, base) in cases {
    let actual = Path::new(target).relative(Path::new(base));
    let expected = oracle_with_cwd(Path::new(target), Path::new(base), &cwd);
    assert_eq!(actual.as_os_str(), expected.as_os_str(), "target {target:?}, base {base:?}");
  }
}

fn deep_relative_path(unresolved_parents: usize, suffix: &[&str]) -> PathBuf {
  let mut path = PathBuf::new();
  for _ in 0..unresolved_parents {
    path.push("..");
  }
  for depth in 0..24 {
    path.push(format!("level-{depth}"));
  }
  for component in suffix {
    path.push(component);
  }
  path
}

#[test]
fn deep_equal_parent_hit_remains_lexically_correct_after_inline_storage_spills() {
  let base = deep_relative_path(2, &["chunks"]);
  let target = deep_relative_path(2, &["assets", "index.js"]);
  let expected = Path::new("..").join("assets").join("index.js");

  assert_eq!(target.relative(base).as_os_str(), expected.as_os_str());
}

#[test]
fn deep_unequal_parent_miss_uses_the_cwd_dependent_fallback() {
  let cwd = std::env::current_dir().expect("read cwd");
  let base = deep_relative_path(1, &["chunks"]);
  let target = deep_relative_path(2, &["assets", "index.js"]);
  let expected = oracle_with_cwd(&target, &base, &cwd);

  assert_eq!(target.relative(base).as_os_str(), expected.as_os_str());
}

#[cfg(windows)]
#[test]
fn windows_explicit_cwd_preserves_or_cancels_drive_relative_context() {
  let cwd = Path::new(r"D:\cwd");
  for (target, base, expected) in [
    (r"C:foo", r"C:bar", r"..\foo"),
    (r"C:..\foo", r"c:..\bar", r"..\foo"),
    (r"C:bar\foo", r"C:bar", r"foo"),
    (r"C:dir\C:foo", r"C:.", r"dir\C:foo"),
    (r"C:foo", r"C:foo", r""),
    (r"C:foo", r"C:..\bar", r"C:foo"),
    (r"C:..\foo", r"C:bar", r"C:..\foo"),
    (r"C:foo", r"D:bar", r"C:foo"),
    (r"C:foo", r"C:\bar", r"C:foo"),
    (r"C:\foo", r"C:bar", r"C:\foo"),
  ] {
    let actual = Path::new(target).relative_with(base, cwd);
    assert_eq!(
      actual.as_os_str(),
      Path::new(expected).as_os_str(),
      "target {target:?}, base {base:?}"
    );
  }
}

#[cfg(windows)]
#[test]
fn windows_root_relative_inputs_cancel_the_unknown_drive() {
  for (target, base, expected) in [
    (r"\foo", r"\bar", r"..\foo"),
    (r"\foo", r"\foo", ""),
    (r"\a\.\foo", r"/a//bar\..", "foo"),
    (r"\..\FOO", r"\foo", ""),
  ] {
    let ambient = Path::new(target).relative(base);
    let explicit = Path::new(target).relative_with(base, "not/absolute");
    assert_eq!(ambient.as_os_str(), Path::new(expected).as_os_str());
    assert_eq!(explicit.as_os_str(), Path::new(expected).as_os_str());
  }
}

#[test]
fn cwd_independent_explicit_relative_does_not_validate_unused_cwd() {
  let actual = Path::new("dist/assets/index.js").relative_with("dist/chunks", "not/absolute");
  let expected = Path::new("..").join("assets").join("index.js");
  assert_eq!(actual.as_os_str(), expected.as_os_str());
}

#[test]
fn cwd_dependent_explicit_relative_rejects_nonabsolute_cwd() {
  let panic =
    std::panic::catch_unwind(|| Path::new("../target").relative_with("base", "not/absolute"));
  assert!(panic.is_err());
}
