#![cfg(all(target_os = "wasi", target_env = "p1"))]

use std::path::Path;

use sugar_path::SugarPath;

#[test]
fn ambient_cwd_observes_the_requested_feature_policy() {
  std::env::set_current_dir("/sugar-path-wasi-a").expect("enter the first preopened directory");
  let first = Path::new("src/lib.rs").absolutize().into_owned();
  assert_eq!(first, Path::new("/sugar-path-wasi-a/src/lib.rs"));

  std::env::set_current_dir("/sugar-path-wasi-b").expect("enter the second preopened directory");
  let second = Path::new("src/lib.rs").try_absolutize().unwrap().into_owned();
  let expected = if cfg!(feature = "cached_current_dir") {
    Path::new("/sugar-path-wasi-a/src/lib.rs")
  } else {
    Path::new("/sugar-path-wasi-b/src/lib.rs")
  };
  assert_eq!(second, expected);
}
