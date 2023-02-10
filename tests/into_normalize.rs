use std::path::Path;
use sugar_path::SugarPathBuf;

#[cfg(target_family = "unix")]
#[test]
fn unix() {

    assert_eq!(
        Path::new("/foo/../../../bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("/bar")
    );
    assert_eq!(
        Path::new("a//b//../b")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("a/b")
    );
    assert_eq!(
        Path::new("/foo/../../../bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("/bar")
    );
    assert_eq!(
        Path::new("a//b//./c")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("a/b/c")
    );
    assert_eq!(
        Path::new("a//b//.")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("a/b")
    );
    assert_eq!(
        Path::new("/a/b/c/../../../x/y/z")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("/x/y/z")
    );
    assert_eq!(
        Path::new("///..//./foo/.//bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("/foo/bar")
    );
    assert_eq!(
        Path::new("bar/foo../../")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("bar/")
    );
    assert_eq!(
        Path::new("bar/foo../..")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("bar")
    );
    assert_eq!(
        Path::new("bar/foo../../baz")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("bar/baz")
    );
    assert_eq!(
        Path::new("bar/foo../")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("bar/foo../")
    );
    assert_eq!(
        Path::new("bar/foo..")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("bar/foo..")
    );
    assert_eq!(
        Path::new("../foo../../../bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("../../bar")
    );
    assert_eq!(
        Path::new("../foo../../../bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("../../bar")
    );
    assert_eq!(
        Path::new("../.../.././.../../../bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("../../bar")
    );
    assert_eq!(
        Path::new("../.../.././.../../../bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("../../bar")
    );
    assert_eq!(
        Path::new("../../../foo/../../../bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("../../../../../bar")
    );
    assert_eq!(
        Path::new("../../../foo/../../../bar/../../")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("../../../../../../")
    );
    assert_eq!(
        Path::new("../foobar/barfoo/foo/../../../bar/../../")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("../../")
    );
    assert_eq!(
        Path::new("../.../../foobar/../../../bar/../../baz")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("../../../../baz")
    );
    assert_eq!(
        Path::new("foo/bar\\baz")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("foo/bar\\baz")
    );
    assert_eq!(
        Path::new("/a/b/c/../../../")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("/")
    );
    assert_eq!(
        Path::new("a/b/c/../../../")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new(".")
    );
    assert_eq!(
        Path::new("a/b/c/../../..")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new(".")
    );

    assert_eq!(
        Path::new("").to_path_buf().into_normalize().as_path(),
        Path::new(".")
    );
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
    assert_eq!(
        Path::new("").to_path_buf().into_normalize().as_path(),
        Path::new(".")
    );
    assert_eq!(
        Path::new("./fixtures///b/../b/c.js")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("fixtures\\b\\c.js")
    );
    assert_eq!(
        Path::new("/foo/../../../bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("\\bar")
    );
    assert_eq!(
        Path::new("a//b//../b")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("a\\b")
    );
    assert_eq!(
        Path::new("a//b//./c")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("a\\b\\c")
    );
    assert_eq!(
        Path::new("//server/share/dir/file.ext")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("\\\\server\\share\\dir\\file.ext")
    );
    assert_eq!(
        Path::new("/foo/../../../bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("\\bar")
    );
    assert_eq!(
        Path::new("/a/b/c/../../../x/y/z")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("\\x\\y\\z")
    );
    assert_eq!(
        Path::new("C:").to_path_buf().into_normalize().as_path(),
        Path::new("C:.")
    );
    assert_eq!(
        Path::new("C:/").to_path_buf().into_normalize().as_path(),
        Path::new("C:\\")
    );
    assert_eq!(
        Path::new("").to_path_buf().into_normalize().as_path(),
        Path::new(".")
    );
    assert_eq!(
        Path::new("c:/ignore")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("c:\\ignore")
    );
    assert_eq!(
        Path::new("C:../a").to_path_buf().into_normalize().as_path(),
        Path::new("C:..\\a")
    );
    assert_eq!(
        Path::new("c:/../a")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("c:\\a")
    );
    assert_eq!(
        Path::new("C:..\\..\\abc\\..\\def")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("C:..\\..\\def")
    );
    assert_eq!(
        Path::new("C:\\..\\..\\abc\\..\\def")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("C:\\def")
    );
    assert_eq!(
        Path::new("C:\\.").to_path_buf().into_normalize().as_path(),
        Path::new("C:\\")
    );

    assert_eq!(
        Path::new("file:stream")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("file:stream")
    );
    assert_eq!(
        Path::new("bar\\foo..\\..\\")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("bar\\")
    );
    assert_eq!(
        Path::new("bar\\foo..\\..\\")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("bar\\")
    );
    assert_eq!(
        Path::new("bar\\foo..\\..")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("bar")
    );
    assert_eq!(
        Path::new("bar\\foo..\\..\\baz")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("bar\\baz")
    );
    assert_eq!(
        Path::new("bar\\foo..\\")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("bar\\foo..\\")
    );
    assert_eq!(
        Path::new("..\\foo..\\..\\..\\bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("..\\..\\bar")
    );
    assert_eq!(
        Path::new("..\\...\\..\\.\\...\\..\\..\\bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("..\\..\\bar")
    );
    assert_eq!(
        Path::new("../../../foo/../../../bar")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("..\\..\\..\\..\\..\\bar")
    );
    assert_eq!(
        Path::new("../../../foo/../../../bar/../../")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("..\\..\\..\\..\\..\\..\\")
    );
    assert_eq!(
        Path::new("../foobar/barfoo/foo/../../../bar/../../")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("..\\..\\")
    );
    assert_eq!(
        Path::new("../.../../foobar/../../../bar/../../baz")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("..\\..\\..\\..\\baz")
    );
    assert_eq!(
        Path::new("foo/bar\\baz")
            .to_path_buf()
            .into_normalize()
            .as_path(),
        Path::new("foo\\bar\\baz")
    );
}
