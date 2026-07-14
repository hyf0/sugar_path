#![cfg(unix)]

use std::{
  borrow::Cow,
  env, fs,
  panic::{AssertUnwindSafe, catch_unwind},
  path::{Path, PathBuf},
  process::Command,
  time::{SystemTime, UNIX_EPOCH},
};

use sugar_path::SugarPath;

const CHILD_ENV: &str = "SUGAR_PATH_CACHED_CWD_FAILURE_ROOT";

fn assert_owned_relative(target: &Path, base: &Path, expected: &Path, context: &str) {
  let fallible = target.try_relative(base).expect("fixture should resolve against cwd");
  assert_eq!(fallible.as_os_str(), expected.as_os_str(), "{context} try");
  assert!(matches!(fallible, Cow::Owned(_)), "{context} try should own");

  let strict = target.relative(base);
  assert_eq!(strict.as_os_str(), expected.as_os_str(), "{context} strict");
  assert!(matches!(strict, Cow::Owned(_)), "{context} strict should own");
}

#[test]
fn failed_cwd_lookup_does_not_poison_later_relative_calls() {
  if let Some(root) = env::var_os(CHILD_ENV) {
    let root = PathBuf::from(root);
    let doomed = root.join("doomed");
    let recovery = root.join("recovery");
    let later = root.join("later");
    let anchor = root.join("anchor");

    fs::remove_dir(&doomed).expect("remove the child's current directory");
    assert!(env::current_dir().is_err());
    assert!(Path::new("recovered.js").try_relative(&anchor).is_err());
    assert!(
      catch_unwind(AssertUnwindSafe(|| Path::new("recovered.js").relative(&anchor))).is_err(),
      "strict relative should panic while cwd is unavailable",
    );

    env::set_current_dir(&recovery).expect("enter the recovery directory");
    let recovery = env::current_dir().expect("read the recovery directory");
    let anchor = recovery.parent().expect("recovery directory has a parent").join("anchor");
    let recovery_name = recovery.file_name().expect("recovery directory has a name");
    assert_owned_relative(
      Path::new("recovered.js"),
      &anchor,
      &PathBuf::from("..").join(recovery_name).join("recovered.js"),
      "first successful cwd lookup after failures",
    );

    env::set_current_dir(&later).expect("enter the later directory");
    let later = env::current_dir().expect("read the later directory");
    let expected_base = if cfg!(feature = "cached_current_dir") { &recovery } else { &later };
    let expected_name = expected_base.file_name().expect("expected base has a name");
    assert_owned_relative(
      Path::new("recovered.js"),
      &anchor,
      &PathBuf::from("..").join(expected_name).join("recovered.js"),
      "failed lookup does not poison cwd policy",
    );
    return;
  }

  let unique = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock is after the Unix epoch")
    .as_nanos();
  let root =
    env::temp_dir().join(format!("sugar-path-cached-cwd-failure-{}-{unique}", std::process::id()));
  let doomed = root.join("doomed");
  fs::create_dir_all(&doomed).expect("create the child's current directory");
  fs::create_dir(root.join("recovery")).expect("create the recovery directory");
  fs::create_dir(root.join("later")).expect("create the later directory");

  let output = Command::new(env::current_exe().expect("find the integration-test executable"))
    .args(["--exact", "failed_cwd_lookup_does_not_poison_later_relative_calls", "--nocapture"])
    .env(CHILD_ENV, &root)
    .current_dir(&doomed)
    .output()
    .expect("run the integration test in a child process");

  if root.exists() {
    fs::remove_dir_all(&root).expect("clean up the child's temporary directories");
  }
  assert!(
    output.status.success(),
    "child test failed\nstdout:\n{}\nstderr:\n{}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr),
  );
}
