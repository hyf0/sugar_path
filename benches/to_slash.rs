use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod fixtures;

use fixtures::{ABSOLUTE_PATHS, FIXTURES};

fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("to_slash", |b| {
    b.iter(|| {
      for fixture in FIXTURES {
        let path = Path::new(fixture);
        let result = black_box(path.to_slash());
        black_box(result);
      }
    })
  });

  c.bench_function("to_slash_lossy", |b| {
    b.iter(|| {
      for fixture in FIXTURES {
        let path = Path::new(fixture);
        let result = black_box(path.to_slash_lossy());
        black_box(result);
      }
    })
  });

  c.bench_function("to_slash_absolute_paths", |b| {
    b.iter(|| {
      for path_str in ABSOLUTE_PATHS {
        let path = Path::new(path_str);
        let result = black_box(path.to_slash());
        black_box(result);
      }
    })
  });

  c.bench_function("to_slash_lossy_absolute_paths", |b| {
    b.iter(|| {
      for path_str in ABSOLUTE_PATHS {
        let path = Path::new(path_str);
        let result = black_box(path.to_slash_lossy());
        black_box(result);
      }
    })
  });

  #[cfg(target_family = "windows")]
  c.bench_function("to_slash_windows_specific", |b| {
    let windows_paths = vec![
      "C:\\Windows\\System32",
      "C:\\Users\\Admin\\Documents\\file.txt",
      "D:\\Projects\\rust\\src\\main.rs",
      "\\\\server\\share\\folder\\document.doc",
      "C:\\Program Files\\Application\\bin",
      "C:\\temp\\cache\\..\\data",
      "file:stream",
      "C:relative\\path",
    ];

    b.iter(|| {
      for path_str in &windows_paths {
        let path = Path::new(path_str);
        let result = black_box(path.to_slash());
        black_box(result);
      }
    })
  });

  #[cfg(target_family = "windows")]
  c.bench_function("to_slash_lossy_windows_specific", |b| {
    let windows_paths = vec![
      "C:\\Windows\\System32",
      "C:\\Users\\Admin\\Documents\\file.txt",
      "D:\\Projects\\rust\\src\\main.rs",
      "\\\\server\\share\\folder\\document.doc",
      "C:\\Program Files\\Application\\bin",
      "C:\\temp\\cache\\..\\data",
      "file:stream",
      "C:relative\\path",
    ];

    b.iter(|| {
      for path_str in &windows_paths {
        let path = Path::new(path_str);
        let result = black_box(path.to_slash_lossy());
        black_box(result);
      }
    })
  });

  c.bench_function("to_slash_mixed_separators", |b| {
    let mixed_paths =
      vec!["foo/bar\\baz", "hello\\world/test", "./foo\\../bar/baz", "C:/Users\\Admin/Documents"];

    b.iter(|| {
      for path_str in &mixed_paths {
        let path = Path::new(path_str);
        let result = black_box(path.to_slash());
        black_box(result);
      }
    })
  });

  c.bench_function("to_slash_deep_nesting", |b| {
    let deep_paths = vec![
      "a/b/c/d/e/f/g/h/i/j/k/l/m/n",
      "/usr/local/lib/python3.9/site-packages/numpy/core/include",
      "./very/long/relative/path/to/some/deeply/nested/file.txt",
    ];

    b.iter(|| {
      for path_str in &deep_paths {
        let path = Path::new(path_str);
        let result = black_box(path.to_slash());
        black_box(result);
      }
    })
  });

  c.bench_function("to_slash_vs_to_slash_lossy", |b| {
    b.iter(|| {
      for fixture in FIXTURES {
        let path = Path::new(fixture);
        let result1 = black_box(path.to_slash());
        let result2 = black_box(path.to_slash_lossy());
        black_box((result1, result2));
      }
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
