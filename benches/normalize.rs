use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod fixtures;

#[cfg(not(target_family = "windows"))]
use fixtures::ALREADY_NORMALIZED_UNIX;
#[cfg(target_family = "windows")]
use fixtures::ALREADY_NORMALIZED_WINDOWS;
use fixtures::{ABSOLUTE_PATHS, FIXTURES};

fn criterion_benchmark(c: &mut Criterion) {
  // Paths that need normalization (existing behavior baseline)
  c.bench_function("normalize_needs_work", |b| {
    b.iter(|| {
      for fixture in FIXTURES {
        black_box(Path::new(fixture).normalize());
      }
    })
  });

  // Paths already in normal form (the Cow::Borrowed fast path target)
  c.bench_function("normalize_already_clean", |b| {
    #[cfg(not(target_family = "windows"))]
    let paths = ALREADY_NORMALIZED_UNIX;
    #[cfg(target_family = "windows")]
    let paths = ALREADY_NORMALIZED_WINDOWS;

    b.iter(|| {
      for fixture in paths {
        black_box(Path::new(fixture).normalize());
      }
    })
  });

  // Already-normalized absolute paths (reuses existing ABSOLUTE_PATHS)
  c.bench_function("normalize_already_clean_absolute", |b| {
    b.iter(|| {
      for fixture in ABSOLUTE_PATHS {
        black_box(Path::new(fixture).normalize());
      }
    })
  });

  // Mixed workload: interleaved clean and needs-work paths
  c.bench_function("normalize_mixed_workload", |b| {
    #[cfg(not(target_family = "windows"))]
    let clean = ALREADY_NORMALIZED_UNIX;
    #[cfg(target_family = "windows")]
    let clean = ALREADY_NORMALIZED_WINDOWS;

    let mixed: Vec<&str> = clean.iter().zip(FIXTURES.iter()).flat_map(|(c, d)| [*c, *d]).collect();

    b.iter(|| {
      for fixture in &mixed {
        black_box(Path::new(fixture).normalize());
      }
    })
  });

  // Short clean paths (isolate fixed-overhead savings)
  c.bench_function("normalize_short_clean", |b| {
    let short_paths = [
      "foo",
      "foo/bar",
      "/foo",
      "/foo/bar",
      "src/main.rs",
      "file.txt",
      "bar",
      "baz/qux",
      "/bar",
      "/bar/baz",
      "tests/unit.rs",
      "image.png",
    ];
    b.iter(|| {
      for fixture in &short_paths {
        black_box(Path::new(fixture).normalize());
      }
    })
  });

  // Deep clean paths (isolate memchr scan cost scaling)
  c.bench_function("normalize_deep_clean", |b| {
    let deep_paths = [
      "a/b/c/d/e/f/g/h/i/j",
      "/usr/local/share/doc/packages/example/tutorials/advanced/chapter1/section2",
      "/home/user/projects/company/backend/services/api/controllers/v2/handlers/auth/login/validate/token/refresh/generate/key/store/cache/data",
      "/level1/level2/level3/level4/level5/level6/level7/level8/level9/level10/level11/level12",
      "p/q/r/s/t/u/v/w/x/y",
      "/opt/data/warehouse/etl/pipelines/transforms/staging/output/validated/reports",
      "/srv/data/projects/org/team/repo/packages/core/src/modules/auth/handlers/v3/internal/process/queue/worker/task",
      "/alpha/bravo/charlie/delta/echo/foxtrot/golf/hotel/india/juliet/kilo/lima",
    ];
    b.iter(|| {
      for fixture in &deep_paths {
        black_box(Path::new(fixture).normalize());
      }
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
