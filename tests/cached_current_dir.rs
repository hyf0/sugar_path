use std::{
  env, fs,
  path::{Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};

use sugar_path::SugarPath;

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

  env::set_current_dir(&second).expect("enter the second temporary directory");
  let second = env::current_dir().expect("read the second temporary directory");
  assert_eq!(Path::new("entry.js").absolutize(), second.join("entry.js"));

  env::set_current_dir(&third).expect("enter the third temporary directory");
  let third = env::current_dir().expect("read the third temporary directory");
  let expected_base = if cfg!(feature = "cached_current_dir") { &second } else { &third };
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
    assert_eq!(
      Path::new(&drive_relative).absolutize().as_ref(),
      oracle.as_path(),
      "drive-relative resolution must bypass the single cached cwd",
    );
  }
}
