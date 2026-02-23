use std::path::PathBuf;
use sugar_path::SugarPath;
mod test_utils;

fn get_cwd() -> PathBuf {
  std::env::current_dir().unwrap()
}

#[cfg(target_family = "unix")]
#[test]
fn unix() {
  assert_eq_str!(pb!("/var/lib", "../", "file/").absolutize(), "/var/file");
  assert_eq!(pb!("a/b/c/", "../../..").absolutize(), get_cwd());
  assert_eq!(pb!(".").absolutize(), get_cwd());
  assert_eq!(pb!().absolutize(), get_cwd());
  assert_eq!(pb!("a").absolutize(), get_cwd().join("a"));
  assert_eq_str!(pb!("/some/dir", ".", "/absolute/").absolutize(), "/absolute");
  assert_eq_str!(
    pb!("/foo/tmp.3/", "../tmp.3/cycles/root.js").absolutize(),
    "/foo/tmp.3/cycles/root.js"
  );
  assert_eq_str!(pb!("/var/lib", "/../", "file/").absolutize(), "/file");
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
  assert_eq_str!(pb!("c:../a").absolutize(), "c:\\a");
  assert_eq_str!(pb!("c:./a").absolutize(), "c:\\a");
  assert_eq!(pb!("a").absolutize(), get_cwd().join("a"));

  assert_eq_str!(pb!("c:/ignore").absolutize(), "c:\\ignore");
  assert_eq_str!(pb!("c:\\some\\file").absolutize(), "c:\\some\\file");
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
