use std::path::PathBuf;

use sugar_path::PathSugar;

fn get_cwd() -> PathBuf {
    std::env::current_dir().unwrap()
}

#[macro_export]
macro_rules! path_buf {
    ( $( $x:expr ),* ) => {
      {
        let mut path_buf = std::path::PathBuf::new();
        $(
          path_buf.push($x);
        )*
        path_buf
      }
    };
}
#[cfg(target_family = "unix")]
#[test]
fn unix() {
    assert_eq!(
        path_buf!("/var/lib", "../", "file/").resolve(),
        path_buf!("/var/file")
    );
    assert_eq!(path_buf!("a/b/c/", "../../..").resolve(), get_cwd());
    assert_eq!(path_buf!(".").resolve(), get_cwd());
    assert_eq!(
        path_buf!("/some/dir", ".", "/absolute/").resolve(),
        path_buf!("/absolute")
    );
    assert_eq!(
        path_buf!("/foo/tmp.3/", "../tmp.3/cycles/root.js").resolve(),
        path_buf!("/foo/tmp.3/cycles/root.js")
    );
    assert_eq!(
        path_buf!("/var/lib", "/../", "file/").resolve(),
        path_buf!("/file")
    );
    assert_eq!(path_buf!().resolve(), get_cwd());
}

#[cfg(target_family = "windows")]
#[test]
#[ignore = "reason"]
fn windows() {
  assert_eq!(path_buf!("c:/blah\\blah", "d:/games", "c:../a"), path_buf!("c:\\blah\\a"));
  assert_eq!(path_buf!("c:/ignore", "d:\\a/b\\c/d", "\\e.exe"), path_buf!("d:\\e.exe"));
  assert_eq!(path_buf!("c:/ignore", "c:/some/file"), path_buf!("c:\\some\\file"));
  assert_eq!(path_buf!("d:/ignore", "d:some/dir//"), path_buf!("d:\\ignore\\some\\dir"));
  assert_eq!(path_buf!("."), get_cwd());
  assert_eq!(path_buf!("//server/share", "..", "relative\\"), path_buf!("\\\\server\\share\\relative"));
  assert_eq!(path_buf!("c:/", "//"), path_buf!("c:\\"));
  assert_eq!(path_buf!("c:/", "//dir"), path_buf!("c:\\dir"));
  assert_eq!(path_buf!("c:/", "//server/share"), path_buf!("\\\\server\\share\\"));
  assert_eq!(path_buf!("c:/", "//server//share"), path_buf!("\\\\server\\share\\"));
  assert_eq!(path_buf!("c:/", "///some//dir"), path_buf!("c:\\some\\dir"));
  assert_eq!(path_buf!("C:\\foo\\tmp.3\\", "..\\tmp.3\\cycles\\root.js"), path_buf!("C:\\foo\\tmp.3\\cycles\\root.js"));
}