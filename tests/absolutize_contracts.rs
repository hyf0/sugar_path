#![cfg(any(target_family = "unix", target_family = "windows"))]

use std::{
  borrow::Cow,
  env,
  path::Path,
};
#[cfg(target_family = "windows")]
use std::path::PathBuf;

use sugar_path::SugarPath;

#[derive(Clone, Copy)]
enum ExpectedCow {
  Borrowed,
  Owned,
}

fn assert_result(
  result: Cow<'_, Path>,
  input: &Path,
  expected: &Path,
  expected_cow: ExpectedCow,
  context: &str,
) {
  assert_eq!(result.as_os_str(), expected.as_os_str(), "{context}");
  match (expected_cow, result) {
    (ExpectedCow::Borrowed, Cow::Borrowed(result)) => {
      assert!(std::ptr::eq(result, input), "{context}: result did not borrow the receiver");
    }
    (ExpectedCow::Owned, Cow::Owned(_)) => {}
    (ExpectedCow::Borrowed, Cow::Owned(_)) => panic!("{context}: expected borrowed result"),
    (ExpectedCow::Owned, Cow::Borrowed(_)) => panic!("{context}: expected owned result"),
  }
}

fn assert_ambient_case(input: &Path, expected: &Path, expected_cow: ExpectedCow, name: &str) {
  assert_result(input.absolutize(), input, expected, expected_cow, &format!("{name} strict"));
  assert_result(
    input.try_absolutize().expect("fixture should resolve against cwd"),
    input,
    expected,
    expected_cow,
    &format!("{name} try"),
  );
}

fn assert_string_receiver(input: String, expected: &Path) {
  let strict = input.absolutize();
  assert_eq!(strict.as_os_str(), expected.as_os_str());
  assert!(matches!(strict, Cow::Owned(_)));

  let fallible = input.try_absolutize().expect("String fixture should resolve against cwd");
  assert_eq!(fallible.as_os_str(), expected.as_os_str());
  assert!(matches!(fallible, Cow::Owned(_)));
}

#[cfg(target_family = "unix")]
#[test]
fn unix_ambient_absolutize_and_try_contract_matrix() {
  let cwd = env::current_dir().expect("read current directory");

  let clean = Path::new("/some/β/file");
  assert_ambient_case(clean, clean, ExpectedCow::Borrowed, "clean absolute");
  assert_ambient_case(
    Path::new("/some/./β/../file/"),
    Path::new("/some/file"),
    ExpectedCow::Owned,
    "dirty absolute",
  );
  assert_ambient_case(Path::new(""), &cwd, ExpectedCow::Owned, "empty");
  assert_ambient_case(Path::new("."), &cwd, ExpectedCow::Owned, "dot");
  assert_ambient_case(
    Path::new("./pkg//β/../file/"),
    &cwd.join("pkg/file"),
    ExpectedCow::Owned,
    "ordinary relative",
  );

  assert_string_receiver("./owned/../file".to_owned(), &cwd.join("file"));
}

#[cfg(target_family = "windows")]
fn current_drive(path: &Path) -> u8 {
  use std::path::{Component, Prefix};

  match path.components().next() {
    Some(Component::Prefix(prefix)) => match prefix.kind() {
      Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => drive,
      other => panic!("expected disk cwd, found {other:?}"),
    },
    other => panic!("expected prefixed cwd, found {other:?}"),
  }
}

#[cfg(target_family = "windows")]
fn preserve_drive_spelling(path: &Path, drive: u8) -> PathBuf {
  use std::{
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
  };

  let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
  if wide.get(1) == Some(&(b':' as u16)) {
    wide[0] = drive as u16;
  } else if wide.get(5) == Some(&(b':' as u16))
    && wide.get(..4) == Some(&[b'\\' as u16, b'\\' as u16, b'?' as u16, b'\\' as u16])
  {
    wide[4] = drive as u16;
  } else {
    panic!("expected disk path, found {path:?}");
  }
  PathBuf::from(OsString::from_wide(&wide))
}

#[cfg(target_family = "windows")]
#[test]
fn windows_ambient_absolutize_and_try_contract_matrix() {
  let cwd = env::current_dir().expect("read current directory");

  let clean = Path::new(r"c:\some\β\file");
  assert_ambient_case(clean, clean, ExpectedCow::Borrowed, "clean absolute");
  assert_ambient_case(
    Path::new(r"c:/some/.\β\../file\"),
    Path::new(r"c:\some\file"),
    ExpectedCow::Owned,
    "dirty absolute",
  );
  assert_ambient_case(Path::new(""), &cwd, ExpectedCow::Owned, "empty");
  assert_ambient_case(Path::new("."), &cwd, ExpectedCow::Owned, "dot");
  assert_ambient_case(
    Path::new(r".\pkg\\β\..\file\"),
    &cwd.join(r"pkg\file"),
    ExpectedCow::Owned,
    "ordinary relative",
  );

  let root_relative = Path::new(r"\pkg\.\β\..\file\");
  let mut expected = cwd.clone();
  expected.push(r"\pkg\file");
  assert_ambient_case(root_relative, &expected, ExpectedCow::Owned, "root relative");

  let drive = current_drive(&cwd).to_ascii_lowercase();
  let drive_relative = format!("{}:folder\\.\\file\\", drive as char);
  let oracle = std::path::absolute(&drive_relative).expect("resolve drive-relative oracle");
  let expected = preserve_drive_spelling(&oracle, drive);
  assert_ambient_case(Path::new(&drive_relative), &expected, ExpectedCow::Owned, "drive relative");

  assert_string_receiver(r".\owned\..\file".to_owned(), &cwd.join("file"));
}
