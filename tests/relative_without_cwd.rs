#![cfg(unix)]

use std::{
  borrow::Cow,
  env, fs,
  panic::{AssertUnwindSafe, catch_unwind},
  path::Path,
  process::Command,
  time::{SystemTime, UNIX_EPOCH},
};

use sugar_path::SugarPath;

const CHILD_ENV: &str = "SUGAR_PATH_RELATIVE_WITHOUT_CWD";

#[test]
fn cwd_independent_relative_inputs_do_not_read_the_current_directory() {
  if let Some(doomed) = env::var_os(CHILD_ENV) {
    fs::remove_dir(&doomed).expect("remove the child's current directory");
    assert!(env::current_dir().is_err());

    let dirty =
      Path::new("./dist/assets/./temp/../index.js").relative(Path::new("dist/./chunks/../chunks"));
    assert_eq!(dirty.as_os_str(), Path::new("../assets/index.js").as_os_str());
    assert!(matches!(dirty, Cow::Owned(_)));

    let target = Path::new("../../dist/assets/index.js");
    let base = Path::new("../../dist/chunks");
    let relative = target.relative(base);
    assert_eq!(relative.as_os_str(), Path::new("../assets/index.js").as_os_str());
    assert!(matches!(relative, Cow::Owned(_)));

    let fallible = target.try_relative(base).expect("equal leading parents do not need cwd");
    assert_eq!(fallible.as_os_str(), Path::new("../assets/index.js").as_os_str());
    assert!(matches!(fallible, Cow::Owned(_)));

    assert!(
      Path::new("../../dist/assets/index.js").try_relative(Path::new("../dist/chunks")).is_err(),
    );
    assert!("../../dist/assets/index.js".try_relative("../dist/chunks").is_err());
    assert!(String::from("../../dist/assets/index.js").try_relative("../dist/chunks").is_err(),);

    let absolute_string = String::from("/workspace/src");
    let absolute_relative =
      absolute_string.try_relative("/workspace").expect("absolute String paths do not need cwd");
    assert_eq!(absolute_relative.as_os_str(), Path::new("src").as_os_str());
    assert!(matches!(absolute_relative, Cow::Borrowed(_)));

    let explicit = Path::new("../../dist/assets/index.js")
      .relative_with(Path::new("../dist/chunks"), Path::new("/"));
    assert_eq!(explicit.as_os_str(), Path::new("../assets/index.js").as_os_str());
    assert!(matches!(explicit, Cow::Owned(_)));

    let cwd_dependent = catch_unwind(AssertUnwindSafe(|| {
      Path::new("../../dist/assets/index.js").relative(Path::new("../dist/chunks"))
    }));
    assert!(cwd_dependent.is_err(), "unequal leading parents must retain the cwd fallback");
    let string_cwd_dependent = catch_unwind(AssertUnwindSafe(|| {
      let input = String::from("../../dist/assets/index.js");
      drop(input.relative("../dist/chunks"));
    }));
    assert!(
      string_cwd_dependent.is_err(),
      "cwd-dependent String relative must preserve the strict panic contract",
    );
    return;
  }

  let unique = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock is after the Unix epoch")
    .as_nanos();
  let doomed =
    env::temp_dir().join(format!("sugar-path-relative-no-cwd-{}-{unique}", std::process::id()));
  fs::create_dir(&doomed).expect("create the child's current directory");

  let output = Command::new(env::current_exe().expect("find the integration-test executable"))
    .args([
      "--exact",
      "cwd_independent_relative_inputs_do_not_read_the_current_directory",
      "--nocapture",
    ])
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
