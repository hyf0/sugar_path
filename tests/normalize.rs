use std::path::Path;
use sugar_path::PathSugar;

#[cfg(target_family = "unix")]
#[test]
fn unix() {
    assert_eq!(
        Path::new("/foo/../../../bar").normalize(),
        Path::new("/bar")
    );
    assert_eq!(Path::new("a//b//../b").normalize(), Path::new("a/b"));
    assert_eq!(
        Path::new("/foo/../../../bar").normalize(),
        Path::new("/bar")
    );
    assert_eq!(Path::new("a//b//./c").normalize(), Path::new("a/b/c"));
    assert_eq!(Path::new("a//b//.").normalize(), Path::new("a/b"));
    assert_eq!(
        Path::new("/a/b/c/../../../x/y/z").normalize(),
        Path::new("/x/y/z")
    );
    assert_eq!(
        Path::new("///..//./foo/.//bar").normalize(),
        Path::new("/foo/bar")
    );
    assert_eq!(Path::new("bar/foo../../").normalize(), Path::new("bar/"));
    assert_eq!(Path::new("bar/foo../..").normalize(), Path::new("bar"));
    assert_eq!(
        Path::new("bar/foo../../baz").normalize(),
        Path::new("bar/baz")
    );
    assert_eq!(Path::new("bar/foo../").normalize(), Path::new("bar/foo../"));
    assert_eq!(Path::new("bar/foo..").normalize(), Path::new("bar/foo.."));
    assert_eq!(
        Path::new("../foo../../../bar").normalize(),
        Path::new("../../bar")
    );
    assert_eq!(
        Path::new("../foo../../../bar").normalize(),
        Path::new("../../bar")
    );
    assert_eq!(
        Path::new("../.../.././.../../../bar").normalize(),
        Path::new("../../bar")
    );
    assert_eq!(
        Path::new("../.../.././.../../../bar").normalize(),
        Path::new("../../bar")
    );
    assert_eq!(
        Path::new("../../../foo/../../../bar").normalize(),
        Path::new("../../../../../bar")
    );
    assert_eq!(
        Path::new("../../../foo/../../../bar/../../").normalize(),
        Path::new("../../../../../../")
    );
    assert_eq!(
        Path::new("../foobar/barfoo/foo/../../../bar/../../").normalize(),
        Path::new("../../")
    );
    assert_eq!(
        Path::new("../.../../foobar/../../../bar/../../baz").normalize(),
        Path::new("../../../../baz")
    );
    assert_eq!(
        Path::new("foo/bar\\baz").normalize(),
        Path::new("foo/bar\\baz")
    );
    assert_eq!(Path::new("/a/b/c/../../../").normalize(), Path::new("/"));
    assert_eq!(Path::new("a/b/c/../../../").normalize(), Path::new("."));
    assert_eq!(Path::new("a/b/c/../../..").normalize(), Path::new("."));

    assert_eq!(Path::new("").normalize(), Path::new("."));
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
    assert_eq!(Path::new("").normalize(), Path::new("."));
    assert_eq!(
        Path::new("./fixtures///b/../b/c.js").normalize(),
        Path::new("fixtures\\b\\c.js")
    );
    assert_eq!(
        Path::new("/foo/../../../bar").normalize(),
        Path::new("\\bar")
    );
    assert_eq!(Path::new("a//b//../b").normalize(), Path::new("a\\b"));
    assert_eq!(Path::new("a//b//./c").normalize(), Path::new("a\\b\\c"));
    assert_eq!(
        Path::new("//server/share/dir/file.ext").normalize(),
        Path::new("\\\\server\\share\\dir\\file.ext")
    );
    assert_eq!(
        Path::new("/foo/../../../bar").normalize(),
        Path::new("\\bar")
    );
    assert_eq!(
        Path::new("/a/b/c/../../../x/y/z").normalize(),
        Path::new("\\x\\y\\z")
    );
    assert_eq!(Path::new("C:").normalize(), Path::new("C:"));
    assert_eq!(Path::new("C:..\\abc").normalize(), Path::new("C:abc"));
    assert_eq!(Path::new("c:../a"), Path::new("c:\\a"));
    assert_eq!(
        Path::new("C:..\\..\\abc\\..\\def").normalize(),
        Path::new("C:def")
    );
    assert_eq!(Path::new("C:\\.").normalize(), Path::new("C:\\"));

    assert_eq!(
        Path::new("file:stream").normalize(),
        Path::new("file:stream")
    );
    assert_eq!(
        Path::new("bar\\foo..\\..\\").normalize(),
        Path::new("bar\\")
    );
    assert_eq!(
        Path::new("bar\\foo..\\..\\").normalize(),
        Path::new("bar\\")
    );
    assert_eq!(Path::new("bar\\foo..\\..").normalize(), Path::new("bar"));
    assert_eq!(
        Path::new("bar\\foo..\\..\\baz").normalize(),
        Path::new("bar\\baz")
    );
    assert_eq!(
        Path::new("bar\\foo..\\").normalize(),
        Path::new("bar\\foo..\\")
    );
    assert_eq!(
        Path::new("..\\foo..\\..\\..\\bar").normalize(),
        Path::new("..\\..\\bar")
    );
    assert_eq!(
        Path::new("..\\...\\..\\.\\...\\..\\..\\bar").normalize(),
        Path::new("..\\..\\bar")
    );
    assert_eq!(
        Path::new("../../../foo/../../../bar").normalize(),
        Path::new("..\\..\\..\\..\\..\\bar")
    );
    assert_eq!(
        Path::new("../../../foo/../../../bar/../../").normalize(),
        Path::new("..\\..\\..\\..\\..\\..\\")
    );
    assert_eq!(
        Path::new("../foobar/barfoo/foo/../../../bar/../../").normalize(),
        Path::new("..\\..\\")
    );
    assert_eq!(
        Path::new("../.../../foobar/../../../bar/../../baz").normalize(),
        Path::new("..\\..\\..\\..\\baz")
    );
    assert_eq!(
        Path::new("foo/bar\\baz").normalize(),
        Path::new("foo\\bar\\baz")
    );
}
