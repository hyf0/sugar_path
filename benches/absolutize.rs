use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod fixtures;

use fixtures::ABSOLUTE_PATHS;

fn criterion_benchmark(c: &mut Criterion) {
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
        black_box(absolute_path.absolutize_with(&cwd));
      }
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
