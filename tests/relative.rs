use std::path::Path;

use sugar_path::SugarPath;

#[cfg(target_family = "unix")]
#[test]
fn unix() {
    let cases = [
        ("/var/lib", "/var", ".."),
        ("/var/lib", "/bin", "../../bin"),
        ("/var/lib", "/var/lib", ""),
        ("/var/lib", "/var/apache", "../apache"),
        ("/var/", "/var/lib", "lib"),
        ("/", "/var/lib", "var/lib"),
        (
            "/foo/test",
            "/foo/test/bar/package.json",
            "bar/package.json",
        ),
        ("/Users/a/web/b/test/mails", "/Users/a/web/b", "../.."),
        ("/foo/bar/baz-quux", "/foo/bar/baz", "../baz"),
        ("/foo/bar/baz", "/foo/bar/baz-quux", "../baz-quux"),
        ("/baz-quux", "/baz", "../baz"),
        ("/baz", "/baz-quux", "../baz-quux"),
        ("/page1/page2/foo", "/", "../../.."),
    ];
    cases.into_iter().for_each(|(to, target, right)| {
        assert_eq!(
            Path::new(target).relative(to),
            Path::new(right),
            "for input target: {} base: {}",
            target,
            to
        );
    });
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
    let cases = [
        ("c:/blah\\blah", "d:/games", "d:\\games"),
        ("c:/aaaa/bbbb", "c:/aaaa", ".."),
        ("c:/aaaa/bbbb", "c:/cccc", "..\\..\\cccc"),
        ("c:/aaaa/bbbb", "c:/aaaa/bbbb", ""),
        ("c:/aaaa/bbbb", "c:/aaaa/cccc", "..\\cccc"),
        ("c:/aaaa/", "c:/aaaa/cccc", "cccc"),
        ("c:/", "c:\\aaaa\\bbbb", "aaaa\\bbbb"),
        ("c:/aaaa/bbbb", "d:\\", "d:\\"),
        ("c:/AaAa/bbbb", "c:/aaaa/bbbb", ""),
        ("c:/aaaaa/", "c:/aaaa/cccc", "..\\aaaa\\cccc"),
        ("c:/aaaaa/", "d:/aaaa/cccc", "d:\\aaaa\\cccc"),
        ("C:\\foo\\bar\\baz\\quux", "C:\\", "..\\..\\..\\.."),
        (
            "C:\\foo\\test",
            "C:\\foo\\test\\bar\\package.json",
            "bar\\package.json",
        ),
        ("C:\\foo\\bar\\baz-quux", "C:\\foo\\bar\\baz", "..\\baz"),
        (
            "C:\\foo\\bar\\baz",
            "C:\\foo\\bar\\baz-quux",
            "..\\baz-quux",
        ),
        ("\\\\foo\\bar\\baz", "C:\\baz", "C:\\baz"),
        ("C:\\baz", "\\\\foo\\bar\\baz", "\\\\foo\\bar\\baz"),
        ("C:\\baz-quux", "C:\\baz", "..\\baz"),
        ("C:\\baz", "C:\\baz-quux", "..\\baz-quux"),
    ];
    cases.into_iter().for_each(|(base, target, right)| {
        assert_eq!(
            Path::new(target).relative(Path::new(base)),
            Path::new(right),
            "for input target: {} base: {}",
            target,
            base
        );
    });
}

#[cfg(target_family = "windows")]
#[test]
fn windows_unc() {
    let cases = [
        ("\\\\foo\\bar", "\\\\foo\\bar\\baz", "baz"),
        ("\\\\foo\\bar\\baz-quux", "\\\\foo\\bar\\baz", "..\\baz"),
        ("\\\\foo\\baz-quux", "\\\\foo\\baz", "\\\\foo\\baz"),
        (
            "\\\\foo\\bar\\baz",
            "\\\\foo\\bar\\baz-quux",
            "..\\baz-quux",
        ),
        ("\\\\foo\\bar\\baz", "\\\\foo\\bar", ".."),
    ];
    cases.into_iter().for_each(|(base, target, right)| {
        assert_eq!(
            Path::new(target).relative(Path::new(base)),
            Path::new(right),
            "for input target: {} base: {}",
            target,
            base
        );
    });
}
