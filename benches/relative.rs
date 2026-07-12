use std::hint::black_box;
use std::path::Path;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod support;

use support::workloads::RELATIVE_CASES;
#[cfg(target_family = "windows")]
use support::workloads::WINDOWS_RELATIVE_ROOT_CASES;

fn bench_relative(criterion: &mut Criterion) {
  let mut group =
    criterion.benchmark_group("relative/borrowed_receiver/natural_result/rolldown_shapes");
  for case in RELATIVE_CASES {
    group.throughput(Throughput::Bytes((case.target.len() + case.base.len()) as u64));
    group.bench_with_input(BenchmarkId::from_parameter(case.name), case, |bencher, case| {
      bencher.iter(|| {
        let target = Path::new(black_box(case.target));
        let base = Path::new(black_box(case.base));
        black_box(target.relative(base))
      });
    });
  }
  group.finish();

  let mut group =
    criterion.benchmark_group("relative/borrowed_receiver/pathbuf_result/rolldown_shapes");
  for case in RELATIVE_CASES {
    group.throughput(Throughput::Bytes((case.target.len() + case.base.len()) as u64));
    group.bench_with_input(BenchmarkId::from_parameter(case.name), case, |bencher, case| {
      bencher.iter(|| {
        let target = Path::new(black_box(case.target));
        let base = Path::new(black_box(case.base));
        black_box(target.relative(base).into_owned())
      });
    });
  }
  group.finish();

  let same = RELATIVE_CASES[0].target;
  let mut group = criterion.benchmark_group("relative/borrowed_receiver/natural_result/special");
  group.throughput(Throughput::Bytes((same.len() * 2) as u64));
  group.bench_function("same_path", |bencher| {
    bencher.iter(|| {
      let path = Path::new(black_box(same));
      black_box(path.relative(path))
    });
  });
  group.finish();

  #[cfg(target_family = "windows")]
  {
    let mut group = criterion
      .benchmark_group("relative/borrowed_receiver/natural_result/windows_inputs_and_roots");
    for case in WINDOWS_RELATIVE_ROOT_CASES {
      group.throughput(Throughput::Bytes((case.target.len() + case.base.len()) as u64));
      group.bench_with_input(BenchmarkId::from_parameter(case.name), case, |bencher, case| {
        bencher.iter(|| {
          let target = Path::new(black_box(case.target));
          let base = Path::new(black_box(case.base));
          black_box(target.relative(base))
        });
      });
    }
    group.finish();
  }
}

criterion_group!(benches, bench_relative);
criterion_main!(benches);
