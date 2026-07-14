use std::{
  borrow::Cow,
  env, fs,
  path::{Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};

use sugar_path::{SugarPath, SugarPathBuf};

fn assert_owned_relative(target: &Path, base: &Path, expected: &Path, context: &str) {
  let strict = target.relative(base);
  assert_eq!(strict.as_os_str(), expected.as_os_str(), "{context} strict");
  assert!(matches!(strict, Cow::Owned(_)), "{context} strict should own");

  let fallible = target.try_relative(base).expect("fixture should resolve against cwd");
  assert_eq!(fallible.as_os_str(), expected.as_os_str(), "{context} try");
  assert!(matches!(fallible, Cow::Owned(_)), "{context} try should own");
}

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

#[test]
fn current_directory_is_cached_only_when_requested() {
  let original = env::current_dir().expect("read the original current directory");
  let unique = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock is after the Unix epoch")
    .as_nanos();
  let root = env::temp_dir().join(format!("sugar-path-cwd-{}-{unique}", std::process::id()));
  let first = root.join("first");
  let second = root.join("second");
  let third = root.join("third");
  fs::create_dir_all(&first).expect("create the first temporary directory");
  fs::create_dir_all(&second).expect("create the second temporary directory");
  fs::create_dir_all(&third).expect("create the third temporary directory");
  let _guard = CurrentDirGuard { original, cleanup: root };

  env::set_current_dir(&first).expect("enter the first temporary directory");
  let first = env::current_dir().expect("read the first temporary directory");
  let dirty_absolute = first.join("unused").join("..").join("absolute.js");
  assert_eq!(dirty_absolute.absolutize(), first.join("absolute.js"));
  assert_owned_relative(
    Path::new("pkg/assets/./index.js"),
    Path::new("pkg/chunks"),
    &PathBuf::from("..").join("assets").join("index.js"),
    "cwd-independent relative",
  );

  env::set_current_dir(&second).expect("enter the second temporary directory");
  let second = env::current_dir().expect("read the second temporary directory");
  let root = second.parent().expect("temporary directory has a parent");
  let anchor = root.join("anchor");
  let second_name = second.file_name().expect("second directory has a name");
  let second_target = second.join("absolute-target.js");
  let relative_target = Path::new("../second/relative-target.js");
  let relative_base = Path::new("base");
  assert_owned_relative(
    relative_target,
    relative_base,
    &PathBuf::from("..").join("relative-target.js"),
    "unequal-parent relative pair initializes cwd",
  );
  assert_owned_relative(
    Path::new("entry.js"),
    &anchor,
    &PathBuf::from("..").join(second_name).join("entry.js"),
    "relative receiver initializes cwd",
  );
  assert_owned_relative(
    &second_target,
    Path::new("base"),
    &PathBuf::from("..").join("absolute-target.js"),
    "relative base uses initialized cwd",
  );
  assert_eq!(Path::new("entry.js").absolutize(), second.join("entry.js"));

  env::set_current_dir(&third).expect("enter the third temporary directory");
  let third = env::current_dir().expect("read the third temporary directory");
  let expected_base = if cfg!(feature = "cached_current_dir") { &second } else { &third };
  let expected_base_name = expected_base.file_name().expect("expected base has a name");
  let expected_relative_pair = if cfg!(feature = "cached_current_dir") {
    PathBuf::from("..").join("relative-target.js")
  } else {
    PathBuf::from("..").join("..").join(second_name).join("relative-target.js")
  };
  assert_owned_relative(
    relative_target,
    relative_base,
    &expected_relative_pair,
    "unequal-parent relative pair observes cwd policy",
  );
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
    PathBuf::from("..").join("..").join(second_name).join("relative-target.js").as_os_str(),
    "explicit cwd must bypass ambient caching",
  );
  assert!(matches!(explicit, Cow::Owned(_)), "explicit unequal-parent result should own");
  assert_owned_relative(
    Path::new("entry.js"),
    &anchor,
    &PathBuf::from("..").join(expected_base_name).join("entry.js"),
    "relative receiver observes cwd policy",
  );
  let expected_from_relative_base = if cfg!(feature = "cached_current_dir") {
    PathBuf::from("..").join("absolute-target.js")
  } else {
    PathBuf::from("..").join("..").join(second_name).join("absolute-target.js")
  };
  assert_owned_relative(
    &second_target,
    Path::new("base"),
    &expected_from_relative_base,
    "relative base observes cwd policy",
  );
  assert_eq!(Path::new("entry.js").absolutize(), expected_base.join("entry.js"));

  #[cfg(target_family = "windows")]
  {
    let drive = match third.components().next() {
      Some(std::path::Component::Prefix(prefix)) => match prefix.kind() {
        std::path::Prefix::Disk(drive) => drive as char,
        other => panic!("expected a disk cwd, found {other:?}"),
      },
      other => panic!("expected a prefixed Windows cwd, found {other:?}"),
    };
    let drive_relative = format!("{drive}:drive-entry.js");
    let oracle = std::path::absolute(&drive_relative).expect("resolve the drive-relative oracle");
    assert_eq!(oracle, third.join("drive-entry.js"));
    assert_eq!(
      Path::new(&drive_relative).absolutize().as_ref(),
      oracle.as_path(),
      "drive-relative resolution must bypass the single cached cwd",
    );
    assert_owned_relative(
      Path::new(&drive_relative),
      &anchor,
      &PathBuf::from("..").join("third").join("drive-entry.js"),
      "drive-relative receiver bypasses cached cwd",
    );

    let drive_base = format!("{drive}:drive-base");
    let base_oracle = std::path::absolute(&drive_base).expect("resolve drive-relative base oracle");
    assert_eq!(base_oracle, third.join("drive-base"));
    assert_owned_relative(
      &second_target,
      Path::new(&drive_base),
      &PathBuf::from("..").join("..").join("second").join("absolute-target.js"),
      "drive-relative base bypasses cached cwd",
    );
  }
}
