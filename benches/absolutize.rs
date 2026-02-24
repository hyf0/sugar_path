use std::borrow::Cow;
use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod fixtures;

use fixtures::{ABSOLUTE_PATHS, DIRTY_ABSOLUTE, RELATIVE_CLEAN};

fn criterion_benchmark(c: &mut Criterion) {
  // Mixed absolute paths
  c.bench_function("absolutize", |b| {
    b.iter(|| {
      for absolute_path in ABSOLUTE_PATHS {
        black_box(absolute_path.absolutize());
      }
    })
  });
  let cwd = std::env::current_dir().unwrap();
  c.bench_function("absolutize_with", |b| {
    b.iter(|| {
      for absolute_path in ABSOLUTE_PATHS {
        black_box(absolute_path.absolutize_with(Cow::Borrowed(cwd.as_path())));
      }
    })
  });

  // Already-absolute, already-clean paths (zero-alloc target after Cow change)
  c.bench_function("absolutize_already_clean_absolute", |b| {
    b.iter(|| {
      for path in ABSOLUTE_PATHS {
        black_box(Path::new(path).absolutize());
      }
    })
  });
  c.bench_function("absolutize_with_already_clean_absolute", |b| {
    b.iter(|| {
      for path in ABSOLUTE_PATHS {
        black_box(Path::new(path).absolutize_with(Cow::Borrowed(cwd.as_path())));
      }
    })
  });

  // Relative clean paths (always allocates — control group)
  c.bench_function("absolutize_relative_paths", |b| {
    b.iter(|| {
      for path in RELATIVE_CLEAN {
        black_box(Path::new(path).absolutize());
      }
    })
  });
  c.bench_function("absolutize_with_relative_paths", |b| {
    b.iter(|| {
      for path in RELATIVE_CLEAN {
        black_box(Path::new(path).absolutize_with(Cow::Borrowed(cwd.as_path())));
      }
    })
  });

  // Dirty absolute paths (needs normalization — always allocates)
  c.bench_function("absolutize_dirty_absolute", |b| {
    b.iter(|| {
      for path in DIRTY_ABSOLUTE {
        black_box(Path::new(path).absolutize());
      }
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
