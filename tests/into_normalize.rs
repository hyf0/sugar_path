use sugar_path::SugarPathBuf;
mod test_utils;

#[cfg(target_family = "unix")]
#[test]
fn unix() {
    assert_eq!(pb!("/foo/../../../bar").into_normalize(), pb!("/bar"));
    assert_eq!(pb!("a//b//../b").into_normalize(), pb!("a/b"));
    assert_eq!(pb!("/foo/../../../bar").into_normalize(), pb!("/bar"));
    assert_eq!(pb!("a//b//./c").into_normalize(), pb!("a/b/c"));
    assert_eq!(pb!("a//b//.").into_normalize(), pb!("a/b"));
    assert_eq!(pb!("/a/b/c/../../../x/y/z").into_normalize(), pb!("/x/y/z"));
    assert_eq!(pb!("///..//./foo/.//bar").into_normalize(), pb!("/foo/bar"));
    assert_eq!(pb!("bar/foo../../").into_normalize(), pb!("bar/"));
    assert_eq!(pb!("bar/foo../..").into_normalize(), pb!("bar"));
    assert_eq!(pb!("bar/foo../../baz").into_normalize(), pb!("bar/baz"));
    assert_eq!(pb!("bar/foo../").into_normalize(), pb!("bar/foo../"));
    assert_eq!(pb!("bar/foo..").into_normalize(), pb!("bar/foo.."));
    assert_eq!(pb!("../foo../../../bar").into_normalize(), pb!("../../bar"));
    assert_eq!(pb!("../foo../../../bar").into_normalize(), pb!("../../bar"));
    assert_eq!(
        pb!("../.../.././.../../../bar").into_normalize(),
        pb!("../../bar")
    );
    assert_eq!(
        pb!("../.../.././.../../../bar").into_normalize(),
        pb!("../../bar")
    );
    assert_eq!(
        pb!("../../../foo/../../../bar").into_normalize(),
        pb!("../../../../../bar")
    );
    assert_eq!(
        pb!("../../../foo/../../../bar/../../").into_normalize(),
        pb!("../../../../../../")
    );
    assert_eq!(
        pb!("../foobar/barfoo/foo/../../../bar/../../").into_normalize(),
        pb!("../../")
    );
    assert_eq!(
        pb!("../.../../foobar/../../../bar/../../baz").into_normalize(),
        pb!("../../../../baz")
    );
    assert_eq!(pb!("foo/bar\\baz").into_normalize(), pb!("foo/bar\\baz"));
    assert_eq!(pb!("/a/b/c/../../../").into_normalize(), pb!("/"));
    assert_eq!(pb!("a/b/c/../../../").into_normalize(), pb!("."));
    assert_eq!(pb!("a/b/c/../../..").into_normalize(), pb!("."));

    assert_eq!(pb!("").into_normalize(), pb!("."));
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
    assert_eq!(pb!("").into_normalize(), pb!("."));
    assert_eq!(
        pb!("./fixtures///b/../b/c.js").into_normalize(),
        pb!("fixtures\\b\\c.js")
    );
    assert_eq!(pb!("/foo/../../../bar").into_normalize(), pb!("\\bar"));
    assert_eq!(pb!("a//b//../b").into_normalize(), pb!("a\\b"));
    assert_eq!(pb!("a//b//./c").into_normalize(), pb!("a\\b\\c"));
    assert_eq!(
        pb!("//server/share/dir/file.ext").into_normalize(),
        pb!("\\\\server\\share\\dir\\file.ext")
    );
    assert_eq!(pb!("/foo/../../../bar").into_normalize(), pb!("\\bar"));
    assert_eq!(
        pb!("/a/b/c/../../../x/y/z").into_normalize(),
        pb!("\\x\\y\\z")
    );
    assert_eq!(pb!("C:").into_normalize(), pb!("C:."));
    assert_eq!(pb!("C:/").into_normalize(), pb!("C:\\"));
    assert_eq!(pb!("").into_normalize(), pb!("."));
    assert_eq!(pb!("c:/ignore").into_normalize(), pb!("c:\\ignore"));
    assert_eq!(pb!("C:../a").into_normalize(), pb!("C:..\\a"));
    assert_eq!(pb!("c:/../a").into_normalize(), pb!("c:\\a"));
    assert_eq!(
        pb!("C:..\\..\\abc\\..\\def").into_normalize(),
        pb!("C:..\\..\\def")
    );
    assert_eq!(
        pb!("C:\\..\\..\\abc\\..\\def").into_normalize(),
        pb!("C:\\def")
    );
    assert_eq!(pb!("C:\\.").into_normalize(), pb!("C:\\"));

    assert_eq!(pb!("file:stream").into_normalize(), pb!("file:stream"));
    assert_eq!(pb!("bar\\foo..\\..\\").into_normalize(), pb!("bar\\"));
    assert_eq!(pb!("bar\\foo..\\..\\").into_normalize(), pb!("bar\\"));
    assert_eq!(pb!("bar\\foo..\\..").into_normalize(), pb!("bar"));
    assert_eq!(pb!("bar\\foo..\\..\\baz").into_normalize(), pb!("bar\\baz"));
    assert_eq!(pb!("bar\\foo..\\").into_normalize(), pb!("bar\\foo..\\"));
    assert_eq!(
        pb!("..\\foo..\\..\\..\\bar").into_normalize(),
        pb!("..\\..\\bar")
    );
    assert_eq!(
        pb!("..\\...\\..\\.\\...\\..\\..\\bar").into_normalize(),
        pb!("..\\..\\bar")
    );
    assert_eq!(
        pb!("../../../foo/../../../bar").into_normalize(),
        pb!("..\\..\\..\\..\\..\\bar")
    );
    assert_eq!(
        pb!("../../../foo/../../../bar/../../").into_normalize(),
        pb!("..\\..\\..\\..\\..\\..\\")
    );
    assert_eq!(
        pb!("../foobar/barfoo/foo/../../../bar/../../").into_normalize(),
        pb!("..\\..\\")
    );
    assert_eq!(
        pb!("../.../../foobar/../../../bar/../../baz").into_normalize(),
        pb!("..\\..\\..\\..\\baz")
    );
    assert_eq!(pb!("foo/bar\\baz").into_normalize(), pb!("foo\\bar\\baz"));
}
