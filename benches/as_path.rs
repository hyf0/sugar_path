use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod support;

use support::workloads::ROLLDOWN_PATHS;

fn bench_as_path(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("as_path/rolldown_corpus");
  group.throughput(Throughput::Elements(ROLLDOWN_PATHS.len() as u64));

  group.bench_function("str", |bencher| {
    bencher.iter(|| {
      for case in black_box(ROLLDOWN_PATHS) {
        black_box(black_box(case.path).as_path());
      }
    });
  });

  let owned: Vec<String> = ROLLDOWN_PATHS.iter().map(|case| case.path.to_owned()).collect();
  group.bench_function("string", |bencher| {
    bencher.iter(|| {
      for path in black_box(&owned) {
        black_box(black_box(path).as_path());
      }
    });
  });

  group.bench_function("std_path_new", |bencher| {
    bencher.iter(|| {
      for case in black_box(ROLLDOWN_PATHS) {
        black_box(Path::new(black_box(case.path)));
      }
    });
  });

  group.finish();
}

criterion_group!(benches, bench_as_path);
criterion_main!(benches);
