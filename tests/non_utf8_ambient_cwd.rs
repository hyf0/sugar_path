#![cfg(target_os = "linux")]

use std::{
  env,
  ffi::OsString,
  fs,
  os::unix::ffi::{OsStrExt, OsStringExt},
  path::{Path, PathBuf},
  process::Command,
  time::{SystemTime, UNIX_EPOCH},
};

use sugar_path::SugarPath;

const CHILD_ENV: &str = "SUGAR_PATH_NON_UTF8_AMBIENT_CWD_CHILD";
const NON_UTF8_COMPONENT: &[u8] = b"non_utf8-\x80";

fn assert_exact_path(actual: &Path, expected: &Path, context: &str) {
  assert_eq!(
    actual.as_os_str().as_bytes(),
    expected.as_os_str().as_bytes(),
    "{context} must preserve native cwd bytes",
  );
}

#[test]
fn non_utf8_ambient_cwd_is_preserved() {
  if env::var_os(CHILD_ENV).is_some() {
    let non_utf8_component = OsString::from_vec(NON_UTF8_COMPONENT.to_vec());
    let cwd = env::current_dir().expect("read the child's current directory");
    assert!(
      cwd
        .as_os_str()
        .as_bytes()
        .windows(NON_UTF8_COMPONENT.len())
        .any(|window| window == NON_UTF8_COMPONENT),
      "test setup must enter the non-UTF-8 cwd",
    );
    let root = cwd
      .parent()
      .and_then(Path::parent)
      .expect("the child cwd has the temporary root as a grandparent");

    let input = Path::new("pkg/entry.js");
    let expected_absolute = cwd.join(input);
    assert_exact_path(input.absolutize().as_ref(), &expected_absolute, "absolutize");
    assert_exact_path(
      input.try_absolutize().expect("resolve against the native cwd").as_ref(),
      &expected_absolute,
      "try_absolutize",
    );

    let target = Path::new("target.js");
    let base = root.join("anchor");
    let expected_relative = PathBuf::from("..").join(&non_utf8_component).join("leaf").join(target);
    assert_exact_path(target.relative(&base).as_ref(), &expected_relative, "relative");
    assert_exact_path(
      target.try_relative(&base).expect("resolve relative paths against the native cwd").as_ref(),
      &expected_relative,
      "try_relative",
    );
    return;
  }

  let unique = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock is after the Unix epoch")
    .as_nanos();
  let root =
    env::temp_dir().join(format!("sugar-path-non-utf8-cwd-{}-{unique}", std::process::id()));
  let cwd = root.join(OsString::from_vec(NON_UTF8_COMPONENT.to_vec())).join("leaf");
  fs::create_dir_all(&cwd).expect("create the non-UTF-8 child cwd");

  let output = Command::new(env::current_exe().expect("find the integration-test executable"))
    .args(["--exact", "non_utf8_ambient_cwd_is_preserved", "--nocapture"])
    .env(CHILD_ENV, "1")
    .current_dir(&cwd)
    .output()
    .expect("run the integration test in a child process");

  if root.exists() {
    fs::remove_dir_all(&root).expect("clean up the non-UTF-8 child cwd");
  }
  assert!(
    output.status.success(),
    "child test failed\nstdout:\n{}\nstderr:\n{}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr),
  );
}
