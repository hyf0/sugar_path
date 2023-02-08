use std::path::PathBuf;

use sugar_path::SugarPath;

fn get_cwd() -> PathBuf {
    std::env::current_dir().unwrap()
}

#[macro_export]
macro_rules! path_buf {
    ( $( $x:expr ),* ) => {
      {
        #[allow(unused_mut)]
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
        path_buf!("/var/lib", "../", "file/").absolutize(),
        path_buf!("/var/file")
    );
    assert_eq!(path_buf!("a/b/c/", "../../..").absolutize(), get_cwd());
    assert_eq!(path_buf!(".").absolutize(), get_cwd());
    assert_eq!(path_buf!().absolutize(), get_cwd());
    assert_eq!(path_buf!("a").absolutize(), get_cwd().join("a"));
    assert_eq!(
        path_buf!("/some/dir", ".", "/absolute/").absolutize(),
        path_buf!("/absolute")
    );
    assert_eq!(
        path_buf!("/foo/tmp.3/", "../tmp.3/cycles/root.js").absolutize(),
        path_buf!("/foo/tmp.3/cycles/root.js")
    );
    assert_eq!(
        path_buf!("/var/lib", "/../", "file/").absolutize(),
        path_buf!("/file")
    );
    assert_eq!(path_buf!().absolutize(), get_cwd());
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
    assert_eq!(path_buf!(".").resolve(), get_cwd());
    assert_eq!(path_buf!("").resolve(), get_cwd());
    assert_eq!(path_buf!("c:../a").resolve(), path_buf!("c://a"));
    assert_eq!(path_buf!("c:./a").resolve(), path_buf!("c://a"));
    assert_eq!(path_buf!("a").resolve(), get_cwd().join("a"));

    assert_eq!(path_buf!("c:/ignore").resolve(), path_buf!("c:\\ignore"));
    assert_eq!(
        path_buf!("c:\\some\\file").resolve(),
        path_buf!("c:\\some\\file")
    );
    assert_eq!(
        path_buf!("some/dir//").resolve(),
        get_cwd().join("some").join("dir")
    );
    assert_eq!(
        path_buf!("//server/share", "..", "relative\\").resolve(),
        get_cwd().join(path_buf!("\\\\server\\share\\relative"))
    );
    {
        let mut right = get_cwd();
        right.pop();
        right = right.join(path_buf!("tmp.3\\cycles\\root.js"));
        assert_eq!(path_buf!("..\\tmp.3\\cycles\\root.js").resolve(), right);
    }
}
