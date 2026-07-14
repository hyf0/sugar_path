use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use sugar_path::SugarPath;

fn assert_cow_result(
  result: Cow<'_, str>,
  source: &str,
  expected: &str,
  expect_borrowed: bool,
  receiver: &str,
  policy: &str,
) {
  assert_eq!(result, expected, "{receiver}/{policy}: input {source:?}");

  match result {
    Cow::Borrowed(actual) => {
      assert!(expect_borrowed, "{receiver}/{policy} unexpectedly borrowed: input {source:?}");
      assert_eq!(actual.as_ptr(), source.as_ptr(), "{receiver}/{policy}: input {source:?}");
      assert_eq!(actual.len(), source.len(), "{receiver}/{policy}: input {source:?}");
    }
    Cow::Owned(_) => {
      assert!(!expect_borrowed, "{receiver}/{policy} unexpectedly allocated: input {source:?}");
    }
  }
}

fn assert_policies<'a>(
  receiver: &str,
  source: &'a str,
  strict: Cow<'a, str>,
  fallible: Option<Cow<'a, str>>,
  lossy: Cow<'a, str>,
  expected: &str,
  expect_borrowed: bool,
) {
  assert_cow_result(strict, source, expected, expect_borrowed, receiver, "strict");
  assert_cow_result(
    fallible.expect("the fixture is valid UTF-8"),
    source,
    expected,
    expect_borrowed,
    receiver,
    "try",
  );
  assert_cow_result(lossy, source, expected, expect_borrowed, receiver, "lossy");
}

fn assert_valid_case(input: &str, expected: &str, expect_borrowed: bool) {
  assert_policies(
    "str",
    input,
    input.to_slash(),
    input.try_to_slash(),
    input.to_slash_lossy(),
    expected,
    expect_borrowed,
  );

  let string = input.to_owned();
  assert_policies(
    "String",
    &string,
    string.to_slash(),
    string.try_to_slash(),
    string.to_slash_lossy(),
    expected,
    expect_borrowed,
  );

  let path = Path::new(input);
  let source = path.to_str().expect("the fixture is valid UTF-8");
  assert_policies(
    "Path",
    source,
    path.to_slash(),
    path.try_to_slash(),
    path.to_slash_lossy(),
    expected,
    expect_borrowed,
  );

  let path_buf = PathBuf::from(input);
  let source = path_buf.to_str().expect("the fixture is valid UTF-8");
  assert_policies(
    "PathBuf",
    source,
    path_buf.to_slash(),
    path_buf.try_to_slash(),
    path_buf.to_slash_lossy(),
    expected,
    expect_borrowed,
  );
}

#[cfg(target_family = "unix")]
#[test]
fn unix_valid_receiver_and_policy_matrix_is_exact_and_borrowed() {
  let cases = [
    ("hello/world", "hello/world"),
    (r"/root/./β/../tail//literal\name//", r"/root/./β/../tail//literal\name//"),
  ];

  for (input, expected) in cases {
    assert_valid_case(input, expected, true);
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_valid_receiver_and_policy_matrix_preserves_spelling() {
  let cases = [
    (r"c:hello\world\", "c:hello/world/", false),
    (r"C:\hello\world\", "C:/hello/world/", false),
    (
      r"\\server\share\.\β\..\tail\\foreign/name\\",
      "//server/share/./β/../tail//foreign/name//",
      false,
    ),
    (
      "//server/share/./β/../tail//foreign/name//",
      "//server/share/./β/../tail//foreign/name//",
      true,
    ),
  ];

  for (input, expected, expect_borrowed) in cases {
    assert_valid_case(input, expected, expect_borrowed);
  }
}
