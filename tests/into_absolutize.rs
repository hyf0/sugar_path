use std::path::PathBuf;
use sugar_path::SugarPathBuf;
mod test_utils;

fn get_cwd() -> PathBuf {
    std::env::current_dir().unwrap()
}

#[cfg(target_family = "unix")]
#[test]
fn unix() {
    assert_eq!(
        pb!("/var/lib", "../", "file/").into_absolutize(),
        pb!("/var/file")
    );
    assert_eq!(pb!("a/b/c/", "../../..").into_absolutize(), get_cwd());
    assert_eq!(pb!(".").into_absolutize(), get_cwd());
    assert_eq!(pb!().into_absolutize(), get_cwd());
    assert_eq!(pb!("a").into_absolutize(), get_cwd().join("a"));
    assert_eq!(
        pb!("/some/dir", ".", "/absolute/").into_absolutize(),
        pb!("/absolute")
    );
    assert_eq!(
        pb!("/foo/tmp.3/", "../tmp.3/cycles/root.js").into_absolutize(),
        pb!("/foo/tmp.3/cycles/root.js")
    );
    assert_eq!(pb!("/var/lib", "/../", "file/").into_absolutize(), pb!("/file"));
    assert_eq!(pb!().into_absolutize(), get_cwd());
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
    assert_eq!(pb!(".").into_absolutize(), get_cwd());
    assert_eq!(pb!("").into_absolutize(), get_cwd());
    assert_eq!(pb!("c:../a").into_absolutize(), pb!("c://a"));
    assert_eq!(pb!("c:./a").into_absolutize(), pb!("c://a"));
    assert_eq!(pb!("a").into_absolutize(), get_cwd().join("a"));

    assert_eq!(pb!("c:/ignore").into_absolutize(), pb!("c:\\ignore"));
    assert_eq!(pb!("c:\\some\\file").into_absolutize(), pb!("c:\\some\\file"));
    assert_eq!(
        pb!("some/dir//").into_absolutize(),
        get_cwd().join("some").join("dir")
    );
    assert_eq!(
        pb!("//server/share", "..", "relative\\").into_absolutize(),
        get_cwd().join(pb!("\\\\server\\share\\relative"))
    );
    {
        let mut right = get_cwd();
        right.pop();
        right = right.join(pb!("tmp.3\\cycles\\root.js"));
        assert_eq!(pb!("..\\tmp.3\\cycles\\root.js").into_absolutize(), right);
    }
}
