use std::path::Path;

use criterion::{criterion_group, criterion_main, Criterion};
use sugar_path::SugarPath;

fn normalize() {
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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("normalize", |b| b.iter(normalize));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);