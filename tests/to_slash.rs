#[cfg(target_family = "unix")]
#[test]
fn unix() {
  use sugar_path::SugarPath;

  let cases = [
    ("hello/world", "hello/world"),
    ("hello/world/", "hello/world/"),
    ("/hello/world", "/hello/world"),
    ("/hello/world/", "/hello/world/"),
    ("/hello\\world", "/hello\\world"),
  ];

  for (input, right) in cases {
    assert_eq!(input.to_slash().as_deref(), Some(right), "case: {input:#?}");
    assert_eq!(input.to_slash_lossy(), right, "case: {input:#?}");
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
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
    assert_eq!(input.to_slash().as_deref(), Some(right), "case: {input:#?}");
    assert_eq!(input.to_slash_lossy(), right, "case: {input:#?}");
  }
}
