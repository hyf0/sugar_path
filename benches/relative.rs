use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use sugar_path::SugarPath;

fn criterion_benchmark(c: &mut Criterion) {
  // Common test cases for relative path computation
  let test_cases = vec![
    ("/var/lib", "/var"),
    ("/var/lib", "/bin"),
    ("/var/lib", "/var/lib"),
    ("/var/lib", "/var/apache"),
    ("/var/", "/var/lib"),
    ("/", "/var/lib"),
    ("/foo/test", "/foo/test/bar/package.json"),
    ("/Users/a/web/b/test/mails", "/Users/a/web/b"),
    ("/foo/bar/baz-quux", "/foo/bar/baz"),
    ("/foo/bar/baz", "/foo/bar/baz-quux"),
    ("/home/user/documents", "/home/user/downloads"),
    ("/usr/local/bin", "/usr/share/doc"),
    ("/a/b/c/d/e", "/a/b/f/g/h"),
  ];

  c.bench_function("relative_simple", |b| {
    b.iter(|| {
      for (base, target) in &test_cases {
        let result = black_box(Path::new(target).relative(base));
        black_box(result);
      }
    })
  });

  c.bench_function("relative_deep_nesting", |b| {
    let deep_cases = vec![
      ("/a/b/c/d/e/f/g", "/a/b/c/d/e/f/h"),
      ("/a/b/c/d/e/f/g", "/x/y/z"),
      ("/very/long/path/to/some/deeply/nested/directory", "/very/long/path/to/another/directory"),
      (
        "/usr/local/lib/python3.9/site-packages/numpy",
        "/usr/local/lib/python3.9/site-packages/pandas",
      ),
    ];

    b.iter(|| {
      for (base, target) in &deep_cases {
        let result = black_box(Path::new(target).relative(base));
        black_box(result);
      }
    })
  });

  c.bench_function("relative_with_dots", |b| {
    let dot_cases = vec![
      ("/var/../usr/lib", "/var/../usr/bin"),
      ("/home/./user/../user/docs", "/home/user/downloads"),
      ("/a/b/../c/d", "/a/b/../c/e"),
    ];

    b.iter(|| {
      for (base, target) in &dot_cases {
        let normalized_base = Path::new(base).normalize();
        let normalized_target = Path::new(target).normalize();
        let result = black_box(normalized_target.relative(&normalized_base));
        black_box(result);
      }
    })
  });

  #[cfg(target_family = "windows")]
  c.bench_function("relative_windows_paths", |b| {
    let windows_cases = vec![
      ("C:\\Users\\Admin\\Documents", "C:\\Users\\Admin\\Downloads"),
      ("C:\\Windows\\System32", "C:\\Program Files"),
      ("D:\\Projects\\rust", "D:\\Projects\\python"),
      ("\\\\server\\share\\folder", "\\\\server\\share\\file"),
    ];

    b.iter(|| {
      for (base, target) in &windows_cases {
        let result = black_box(Path::new(target).relative(base));
        black_box(result);
      }
    })
  });

  c.bench_function("relative_same_path", |b| {
    let paths = vec!["/home/user/documents", "/var/log/system", "/usr/local/bin"];

    b.iter(|| {
      for path in &paths {
        // Benchmark relative when base and target are the same
        let result = black_box(Path::new(path).relative(path));
        black_box(result);
      }
    })
  });

  c.bench_function("relative_parent_child", |b| {
    let parent_child = vec![
      ("/parent", "/parent/child"),
      ("/parent/child", "/parent"),
      ("/a/b", "/a/b/c/d"),
      ("/a/b/c/d", "/a/b"),
    ];

    b.iter(|| {
      for (base, target) in &parent_child {
        let result = black_box(Path::new(target).relative(base));
        black_box(result);
      }
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
