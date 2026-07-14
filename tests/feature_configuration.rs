const EXPECTED_CACHED_CURRENT_DIR: &str = "SUGAR_PATH_EXPECT_CACHED_CURRENT_DIR";

#[test]
fn requested_cached_current_dir_configuration_is_active() {
  let Some(expected) = std::env::var_os(EXPECTED_CACHED_CURRENT_DIR) else {
    if cfg!(target_os = "wasi") || std::env::var_os("GITHUB_ACTIONS").is_some() {
      let message = format!(
        "{EXPECTED_CACHED_CURRENT_DIR} must be provided by WASI and GitHub Actions test runners"
      );
      eprintln!("{message}");
      panic!("{message}");
    }
    return;
  };
  let expected = expected.to_str().expect("feature expectation must be valid UTF-8");
  let expected = match expected {
    "0" => false,
    "1" => true,
    value => panic!("{EXPECTED_CACHED_CURRENT_DIR} must be 0 or 1, found {value:?}"),
  };

  assert_eq!(
    cfg!(feature = "cached_current_dir"),
    expected,
    "cached_current_dir feature did not match the requested CI configuration",
  );
}
