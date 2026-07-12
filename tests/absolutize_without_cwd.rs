#![cfg(target_family = "unix")]

use std::{
  borrow::Cow,
  env, fs,
  path::Path,
  process::Command,
  time::{SystemTime, UNIX_EPOCH},
};

use sugar_path::SugarPath;

const CHILD_ENV: &str = "SUGAR_PATH_TEST_WITHOUT_CWD";

#[test]
fn absolute_paths_do_not_require_current_directory() {
  if let Some(doomed) = env::var_os(CHILD_ENV) {
    fs::remove_dir(&doomed).expect("remove the child's current directory");
    assert!(env::current_dir().is_err());

    let clean = Path::new("/sugar-path/file.js");
    let clean_output = clean.absolutize();
    assert!(matches!(clean_output, Cow::Borrowed(_)));
    assert_eq!(clean_output.as_os_str(), clean.as_os_str());
    assert!(clean.try_absolutize().is_ok());

    let dirty = Path::new("/sugar-path/../file.js/");
    let dirty_output = dirty.absolutize();
    assert!(matches!(dirty_output, Cow::Owned(_)));
    assert_eq!(dirty_output.as_os_str(), Path::new("/file.js").as_os_str());
    assert!(dirty.try_absolutize().is_ok());

    assert!(Path::new("relative.js").try_absolutize().is_err());
    assert!(std::panic::catch_unwind(|| Path::new("relative.js").absolutize()).is_err());
    return;
  }

  let unique = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock is after the Unix epoch")
    .as_nanos();
  let doomed = env::temp_dir().join(format!("sugar-path-no-cwd-{}-{unique}", std::process::id()));
  fs::create_dir(&doomed).expect("create the child's current directory");

  let output = Command::new(env::current_exe().expect("find the integration-test executable"))
    .args(["--exact", "absolute_paths_do_not_require_current_directory", "--nocapture"])
    .env(CHILD_ENV, &doomed)
    .current_dir(&doomed)
    .output()
    .expect("run the integration test in a child process");

  if doomed.exists() {
    fs::remove_dir(&doomed).expect("clean up the child's current directory");
  }
  assert!(
    output.status.success(),
    "child test failed\nstdout:\n{}\nstderr:\n{}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr),
  );
}
