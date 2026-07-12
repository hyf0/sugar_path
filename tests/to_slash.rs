#[cfg(target_family = "unix")]
#[test]
fn unix() {
  use std::borrow::Cow;
  use sugar_path::SugarPath;

  let cases = [
    ("hello/world", "hello/world"),
    ("hello/world/", "hello/world/"),
    ("/hello/world", "/hello/world"),
    ("/hello/world/", "/hello/world/"),
    ("/hello\\world", "/hello\\world"),
  ];

  for (input, right) in cases {
    let strict = input.to_slash();
    let lossy = input.to_slash_lossy();
    assert!(matches!(strict, Cow::Borrowed(_)), "strict result should borrow: {input:#?}");
    assert!(matches!(lossy, Cow::Borrowed(_)), "lossy result should borrow: {input:#?}");
    assert_eq!(strict, right, "case: {input:#?}");
    assert_eq!(lossy, right, "case: {input:#?}");
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
  use std::borrow::Cow;
  use sugar_path::SugarPath;

  let cases = [
    ("hello\\world", "hello/world"),
    ("hello\\world\\", "hello/world/"),
    ("c:hello\\world", "c:hello/world"),
    ("c:hello\\world\\", "c:hello/world/"),
    ("c:\\hello\\world", "c:/hello/world"),
    ("c:\\hello\\world/", "c:/hello/world/"),
  ];

  for (input, right) in cases {
    let strict = input.to_slash();
    let lossy = input.to_slash_lossy();
    assert!(matches!(strict, Cow::Owned(_)), "strict result should own: {input:#?}");
    assert!(matches!(lossy, Cow::Owned(_)), "lossy result should own: {input:#?}");
    assert_eq!(strict, right, "case: {input:#?}");
    assert_eq!(lossy, right, "case: {input:#?}");
  }

  let slash_only = "C:/hello/world";
  assert!(matches!(slash_only.to_slash(), Cow::Borrowed(_)));
  assert!(matches!(slash_only.try_to_slash(), Some(Cow::Borrowed(_))));
  assert!(matches!(slash_only.to_slash_lossy(), Cow::Borrowed(_)));
}
