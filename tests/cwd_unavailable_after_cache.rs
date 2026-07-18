#![cfg(unix)]

use std::{
  borrow::Cow,
  env, fs, io,
  panic::{AssertUnwindSafe, catch_unwind},
  path::{Path, PathBuf},
  process::Command,
  time::{SystemTime, UNIX_EPOCH},
};

use sugar_path::SugarPath;

const CHILD_ENV: &str = "SUGAR_PATH_CWD_UNAVAILABLE_AFTER_CACHE";

fn assert_same_error(actual: &io::Error, expected: &io::Error, context: &str) {
  assert_eq!(actual.kind(), expected.kind(), "{context} error kind");
  assert_eq!(actual.raw_os_error(), expected.raw_os_error(), "{context} raw OS error");
}

fn assert_owned(output: Cow<'_, Path>, expected: &Path, context: &str) {
  assert_eq!(output.as_ref(), expected, "{context} value");
  assert!(matches!(output, Cow::Owned(_)), "{context} should own");
}

#[test]
fn cwd_unavailable_after_successful_lookup_observes_cache_policy() {
  if let Some(root) = env::var_os(CHILD_ENV) {
    let root = PathBuf::from(root);
    let doomed = root.join("doomed");
    assert_eq!(env::current_dir().expect("read the child's initial cwd"), doomed);
    let anchor = root.join("anchor");

    assert_owned(
      Path::new("initial.js").try_absolutize().expect("initialize cwd state"),
      &doomed.join("initial.js"),
      "initial cwd lookup",
    );
    fs::remove_dir(&doomed).expect("remove the child's current directory");
    let cwd_error = env::current_dir().expect_err("the process cwd is unavailable");

    let absolute = Path::new("later.js").try_absolutize();
    let relative = Path::new("later.js").try_relative(&anchor);
    if cfg!(feature = "cached_current_dir") {
      assert_owned(
        absolute.expect("cached absolutize should succeed"),
        &doomed.join("later.js"),
        "cached try_absolutize",
      );
      assert_owned(
        relative.expect("cached relative should succeed"),
        Path::new("../doomed/later.js"),
        "cached try_relative",
      );
      assert_owned(
        Path::new("later.js").absolutize(),
        &doomed.join("later.js"),
        "cached absolutize",
      );
      assert_owned(
        Path::new("later.js").relative(&anchor),
        Path::new("../doomed/later.js"),
        "cached relative",
      );
    } else {
      assert_same_error(
        &absolute.expect_err("default absolutize should fail"),
        &cwd_error,
        "default try_absolutize",
      );
      assert_same_error(
        &relative.expect_err("default relative should fail"),
        &cwd_error,
        "default try_relative",
      );
      assert!(
        catch_unwind(|| Path::new("later.js").absolutize()).is_err(),
        "default absolutize should panic",
      );
      assert!(
        catch_unwind(AssertUnwindSafe(|| Path::new("later.js").relative(&anchor))).is_err(),
        "default relative should panic",
      );
    }
    return;
  }

  let unique = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock is after the Unix epoch")
    .as_nanos();
  let root =
    env::temp_dir().join(format!("sugar-path-cwd-unavailable-{}-{unique}", std::process::id()));
  fs::create_dir_all(root.join("doomed")).expect("create the child cwd");
  let root = fs::canonicalize(root).expect("resolve the temporary root spelling");
  let doomed = root.join("doomed");

  let output = Command::new(env::current_exe().expect("find the integration-test executable"))
    .args([
      "--exact",
      "cwd_unavailable_after_successful_lookup_observes_cache_policy",
      "--nocapture",
    ])
    .env(CHILD_ENV, &root)
    .current_dir(&doomed)
    .output()
    .expect("run the integration test in a child process");

  if root.exists() {
    fs::remove_dir_all(&root).expect("clean up the temporary root");
  }
  assert!(
    output.status.success(),
    "child test failed\nstdout:\n{}\nstderr:\n{}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr),
  );
}
