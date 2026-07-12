use std::path::PathBuf;
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::{borrow::Cow, path::Path};
use sugar_path::SugarPath;
mod test_utils;

fn get_cwd() -> PathBuf {
  std::env::current_dir().unwrap()
}

#[cfg(target_family = "unix")]
#[test]
fn unix() {
  assert_eq_str!("/var/lib/../file/".absolutize(), "/var/file");
  assert_eq!("a/b/c/../../..".absolutize(), get_cwd());
  assert_eq!(".".absolutize(), get_cwd());
  assert_eq!("".absolutize(), get_cwd());
  assert_eq!("a".absolutize(), get_cwd().join("a"));
  assert_eq_str!("/absolute/".absolutize(), "/absolute");
  assert_eq_str!("/foo/tmp.3/../tmp.3/cycles/root.js".absolutize(), "/foo/tmp.3/cycles/root.js");
  assert_eq_str!("/../file/".absolutize(), "/file");
}

#[cfg(target_family = "unix")]
#[test]
fn unix_absolute_inputs_preserve_cow_contract() {
  let clean = Path::new("/some/file");
  let clean_output = clean.absolutize();
  assert!(matches!(clean_output, Cow::Borrowed(_)));
  assert_eq!(clean_output.as_os_str(), clean.as_os_str());

  let dirty = Path::new("/some/../file/");
  let dirty_output = dirty.absolutize();
  assert!(matches!(dirty_output, Cow::Owned(_)));
  assert_eq!(dirty_output.as_os_str(), Path::new("/file").as_os_str());
}

#[test]
fn make_sure_dots_are_resolved() {
  assert_eq!("./main".absolutize(), get_cwd().join("main"));
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
  assert_eq!(".".absolutize(), get_cwd());
  assert_eq!("".absolutize(), get_cwd());
  let cwd = get_cwd();
  let drive = match cwd.components().next() {
    Some(std::path::Component::Prefix(prefix)) => match prefix.kind() {
      std::path::Prefix::Disk(drive) => drive,
      other => panic!("expected a disk cwd, found {other:?}"),
    },
    other => panic!("expected a prefixed Windows cwd, found {other:?}"),
  };
  let drive = (drive as char).to_ascii_lowercase();
  for suffix in ["../a", "./a"] {
    let input = format!("{drive}:{suffix}");
    let actual = input.absolutize().into_owned();
    let oracle = std::path::absolute(&input).expect("resolve the drive-relative oracle");
    assert!(
      actual
        .to_str()
        .expect("valid test path")
        .eq_ignore_ascii_case(oracle.to_str().expect("valid oracle path")),
      "input {input:?}: actual {actual:?}, oracle {oracle:?}",
    );
    assert_eq!(actual.to_str().expect("valid test path").chars().next(), Some(drive));
  }
  assert_eq!("a".absolutize(), get_cwd().join("a"));

  assert_eq_str!("c:/ignore".absolutize(), "c:\\ignore");
  assert_eq_str!("c:\\some\\file".absolutize(), "c:\\some\\file");
  assert_eq!("some/dir//".absolutize(), get_cwd().join("some").join("dir"));
  assert_eq_str!("//server/share/../relative\\".absolutize(), "\\\\server\\share\\relative");
  {
    let mut right = get_cwd();
    right.pop();
    right = right.join(pb!("tmp.3\\cycles\\root.js"));
    assert_eq!("..\\tmp.3\\cycles\\root.js".absolutize(), right);
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_absolute_inputs_preserve_cow_contract() {
  let clean = Path::new("C:\\some\\file");
  let clean_output = clean.absolutize();
  assert!(matches!(clean_output, Cow::Borrowed(_)));
  assert_eq!(clean_output.as_os_str(), clean.as_os_str());

  let dirty = Path::new("c:/some/../file");
  let dirty_output = dirty.absolutize();
  assert!(matches!(dirty_output, Cow::Owned(_)));
  assert_eq!(dirty_output.as_os_str(), Path::new("c:\\file").as_os_str());
}
