#![cfg(target_family = "windows")]

use std::{io, path::Path};

use sugar_path::SugarPath;

fn assert_same_error(actual: &io::Error, expected: &io::Error, context: &str) {
  assert_eq!(actual.kind(), expected.kind(), "{context} error kind");
  assert_eq!(actual.raw_os_error(), expected.raw_os_error(), "{context} raw OS error");
}

#[test]
fn windows_drive_relative_resolution_propagates_native_errors() {
  let invalid = "C:bad\0path";
  let expected = std::path::absolute(invalid).expect_err("an embedded NUL is invalid for WinAPI");
  assert_eq!(expected.kind(), io::ErrorKind::InvalidInput);

  assert_same_error(
    &Path::new(invalid).try_absolutize().expect_err("drive-relative Path should fail"),
    &expected,
    "Path try_absolutize",
  );
  assert_same_error(
    &invalid.try_absolutize().expect_err("drive-relative str should fail"),
    &expected,
    "str try_absolutize",
  );
  let owned = invalid.to_owned();
  assert_same_error(
    &owned.try_absolutize().expect_err("drive-relative String should fail"),
    &expected,
    "String try_absolutize",
  );
  assert!(std::panic::catch_unwind(|| Path::new(invalid).absolutize()).is_err());

  let absolute_base = Path::new(r"C:\base");
  assert_same_error(
    &Path::new(invalid).try_relative(absolute_base).expect_err("drive-relative target should fail"),
    &expected,
    "try_relative target",
  );
  assert!(
    std::panic::catch_unwind(|| Path::new(invalid).relative(absolute_base)).is_err(),
    "strict relative should panic for an invalid drive-relative target",
  );

  let absolute_target = Path::new(r"C:\target");
  assert_same_error(
    &absolute_target.try_relative(invalid).expect_err("drive-relative base should fail"),
    &expected,
    "try_relative base",
  );
  assert!(
    std::panic::catch_unwind(|| absolute_target.relative(invalid)).is_err(),
    "strict relative should panic for an invalid drive-relative base",
  );
}
