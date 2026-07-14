#![cfg(target_family = "windows")]

use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use sugar_path::SugarPath;

fn assert_explicit_case(cwd: &str, input: &str, expected: &str, name: &str) {
  assert!(Path::new(cwd).is_absolute(), "{name}: fixture cwd is not absolute");

  let borrowed = Path::new(input).absolutize_with(Path::new(cwd));
  assert_eq!(borrowed.as_os_str(), Path::new(expected).as_os_str(), "{name} borrowed cwd");
  assert!(matches!(borrowed, Cow::Owned(_)), "{name} borrowed cwd should own");

  let owned = Path::new(input).absolutize_with(PathBuf::from(cwd));
  assert_eq!(owned.as_os_str(), Path::new(expected).as_os_str(), "{name} owned cwd");
  assert!(matches!(owned, Cow::Owned(_)), "{name} owned cwd should own");
}

#[test]
fn windows_root_relative_inputs_use_the_exact_explicit_prefix() {
  let cases = [
    ("disk", r"C:\workspace", r"\pkg\.\temp\..\β\file\", r"C:\pkg\β\file"),
    ("disk root", r"C:\workspace", r"\", r"C:\"),
    ("UNC", r"\\Server\Share\workspace", r"\pkg\..\file\", r"\\Server\Share\file"),
    ("UNC root", r"\\Server\Share\workspace", r"\", r"\\Server\Share\"),
    ("verbatim disk", r"\\?\c:\workspace", r"\pkg\..\file\", r"\\?\c:\file"),
    ("verbatim disk root", r"\\?\c:\workspace", r"\", r"\\?\c:\"),
    (
      "verbatim UNC",
      r"\\?\UNC\Server\Share\workspace",
      r"\pkg\..\file\",
      r"\\?\UNC\Server\Share\file",
    ),
    // A verbatim UNC prefix is already rooted and absolute without an explicit
    // RootDir component, so resolution strips the optional trailing separator.
    ("verbatim UNC root", r"\\?\UNC\Server\Share\workspace", r"\", r"\\?\UNC\Server\Share"),
  ];

  for (name, cwd, input, expected) in cases {
    assert_explicit_case(cwd, input, expected, name);
  }
}
