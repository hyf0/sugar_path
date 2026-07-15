use std::{
  borrow::Cow,
  env, fs,
  path::{Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};

use sugar_path::{SugarPath, SugarPathBuf};

struct CurrentDirGuard {
  original: PathBuf,
  cleanup: PathBuf,
}

impl Drop for CurrentDirGuard {
  fn drop(&mut self) {
    env::set_current_dir(&self.original).expect("restore the original current directory");
    fs::remove_dir_all(&self.cleanup).expect("remove the temporary directories");
  }
}

fn assert_owned_relative(target: &Path, base: &Path, expected: &Path, context: &str) {
  let strict = target.relative(base);
  assert_eq!(strict.as_os_str(), expected.as_os_str(), "{context} strict");
  assert!(matches!(strict, Cow::Owned(_)), "{context} strict should own");

  let fallible = target.try_relative(base).expect("fixture should resolve against cwd");
  assert_eq!(fallible.as_os_str(), expected.as_os_str(), "{context} try");
  assert!(matches!(fallible, Cow::Owned(_)), "{context} try should own");
}

#[test]
fn pure_relative_calls_initialize_and_observe_cwd_policy() {
  let original = env::current_dir().expect("read the original current directory");
  let unique = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock is after the Unix epoch")
    .as_nanos();
  let root =
    env::temp_dir().join(format!("sugar-path-relative-cwd-{}-{unique}", std::process::id()));
  let first = root.join("first");
  let second = root.join("second");
  let third = root.join("third");
  fs::create_dir_all(&first).expect("create the first temporary directory");
  fs::create_dir_all(&second).expect("create the second temporary directory");
  fs::create_dir_all(&third).expect("create the third temporary directory");
  let _guard = CurrentDirGuard { original, cleanup: root };

  env::set_current_dir(&first).expect("enter the first temporary directory");
  assert_owned_relative(
    Path::new("pkg/assets/./index.js"),
    Path::new("pkg/chunks"),
    &PathBuf::from("..").join("assets").join("index.js"),
    "cwd-independent preflight",
  );

  env::set_current_dir(&second).expect("enter the second temporary directory");
  let relative_target = Path::new("../second/relative-target.js");
  let relative_base = Path::new("base");
  let initialized = relative_target
    .try_relative(relative_base)
    .expect("unequal-parent fixture should resolve against cwd");
  assert_eq!(initialized.as_os_str(), PathBuf::from("..").join("relative-target.js").as_os_str(),);
  assert!(matches!(initialized, Cow::Owned(_)), "initial unequal-parent result should own");

  env::set_current_dir(&third).expect("enter the third temporary directory");
  let third = env::current_dir().expect("read the third temporary directory");
  let expected = if cfg!(feature = "cached_current_dir") {
    PathBuf::from("..").join("relative-target.js")
  } else {
    PathBuf::from("..").join("..").join("second").join("relative-target.js")
  };
  assert_owned_relative(
    relative_target,
    relative_base,
    &expected,
    "unequal-parent pair observes cwd policy",
  );
  let string_target = String::from("../second/string-target.js");
  let string_relative = string_target.relative(relative_base);
  let expected_string = if cfg!(feature = "cached_current_dir") {
    PathBuf::from("..").join("string-target.js")
  } else {
    PathBuf::from("..").join("..").join("second").join("string-target.js")
  };
  assert_eq!(string_relative.as_os_str(), expected_string.as_os_str());
  assert!(matches!(string_relative, Cow::Owned(_)), "cwd-resolved String relative should own");

  let expected_slash = if cfg!(feature = "cached_current_dir") {
    "../relative-target.js"
  } else {
    "../../second/relative-target.js"
  };
  assert_eq!(
    relative_target.relative(relative_base).into_owned().into_slash(),
    expected_slash,
    "strict relative-to-slash composition observes cwd policy",
  );
  assert_eq!(
    relative_target
      .try_relative(relative_base)
      .expect("fixture should resolve against cwd")
      .into_owned()
      .into_slash(),
    expected_slash,
    "fallible relative-to-slash composition observes cwd policy",
  );

  let explicit = relative_target.relative_with(relative_base, &third);
  assert_eq!(
    explicit.as_os_str(),
    PathBuf::from("..").join("..").join("second").join("relative-target.js").as_os_str(),
    "explicit cwd must bypass ambient caching",
  );
  assert!(matches!(explicit, Cow::Owned(_)), "explicit unequal-parent result should own");
}
