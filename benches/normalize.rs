use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod fixtures;

use fixtures::FIXTURES;

fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("normalize", |b| {
    b.iter(|| {
      for fixture in FIXTURES {
        black_box(Path::new(fixture).normalize());
      }
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
