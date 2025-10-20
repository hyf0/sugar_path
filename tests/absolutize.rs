use std::path::PathBuf;
use sugar_path::SugarPath;
mod test_utils;

fn get_cwd() -> PathBuf {
  std::env::current_dir().unwrap()
}

#[cfg(target_family = "unix")]
#[test]
fn unix() {
  assert_eq!(pb!("/var/lib", "../", "file/").absolutize(), pb!("/var/file"));
  assert_eq!(pb!("a/b/c/", "../../..").absolutize(), get_cwd());
  assert_eq!(pb!(".").absolutize(), get_cwd());
  assert_eq!(pb!().absolutize(), get_cwd());
  assert_eq!(pb!("a").absolutize(), get_cwd().join("a"));
  assert_eq!(pb!("/some/dir", ".", "/absolute/").absolutize(), pb!("/absolute"));
  assert_eq!(
    pb!("/foo/tmp.3/", "../tmp.3/cycles/root.js").absolutize(),
    pb!("/foo/tmp.3/cycles/root.js")
  );
  assert_eq!(pb!("/var/lib", "/../", "file/").absolutize(), pb!("/file"));
  assert_eq!(pb!().absolutize(), get_cwd());
}

#[test]
fn make_sure_dots_are_resolved() {
  assert!(!get_cwd().join("./main").normalize().display().to_string().contains('.'));
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
  assert_eq!(pb!(".").absolutize(), get_cwd());
  assert_eq!(pb!("").absolutize(), get_cwd());
  assert_eq!(pb!("c:../a").absolutize(), pb!("c://a"));
  assert_eq!(pb!("c:./a").absolutize(), pb!("c://a"));
  assert_eq!(pb!("a").absolutize(), get_cwd().join("a"));

  assert_eq!(pb!("c:/ignore").absolutize(), pb!("c:\\ignore"));
  assert_eq!(pb!("c:\\some\\file").absolutize(), pb!("c:\\some\\file"));
  assert_eq!(pb!("some/dir//").absolutize(), get_cwd().join("some").join("dir"));
  assert_eq!(
    pb!("//server/share", "..", "relative\\").absolutize(),
    get_cwd().join(pb!("\\\\server\\share\\relative"))
  );
  {
    let mut right = get_cwd();
    right.pop();
    right = right.join(pb!("tmp.3\\cycles\\root.js"));
    assert_eq!(pb!("..\\tmp.3\\cycles\\root.js").absolutize(), right);
  }
}
