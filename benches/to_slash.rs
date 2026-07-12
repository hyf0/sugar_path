use std::hint::black_box;
use std::path::{Path, PathBuf};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use sugar_path::{SugarPath, SugarPathBuf};

mod support;

use support::workloads::ROLLDOWN_PATHS;
#[cfg(target_family = "windows")]
use support::workloads::WINDOWS_SLASH_CASES;
#[cfg(any(unix, windows))]
use support::workloads::invalid_unicode_path;

fn bench_to_slash(criterion: &mut Criterion) {
  let mut group =
    criterion.benchmark_group("to_slash/borrowed_receiver/natural_result/rolldown_paths");
  for case in ROLLDOWN_PATHS {
    group.throughput(Throughput::Bytes(case.path.len() as u64));
    group.bench_with_input(BenchmarkId::from_parameter(case.name), case, |bencher, case| {
      bencher.iter(|| {
        let path = Path::new(black_box(case.path));
        black_box(path.to_slash())
      });
    });
  }
  group.finish();

  let mut group =
    criterion.benchmark_group("to_slash_lossy/borrowed_receiver/natural_result/rolldown_paths");
  for case in ROLLDOWN_PATHS {
    group.throughput(Throughput::Bytes(case.path.len() as u64));
    group.bench_with_input(BenchmarkId::from_parameter(case.name), case, |bencher, case| {
      bencher.iter(|| {
        let path = Path::new(black_box(case.path));
        black_box(path.to_slash_lossy())
      });
    });
  }
  group.finish();

  let owned_case = ROLLDOWN_PATHS[2];
  let mut group = criterion.benchmark_group("slash/owned_input");
  group.throughput(Throughput::Bytes(owned_case.path.len() as u64));
  group.bench_function("borrowed_receiver/string_result", |bencher| {
    bencher.iter_batched(
      || PathBuf::from(owned_case.path),
      |input| black_box(black_box(input.as_path()).to_slash().into_owned()),
      BatchSize::SmallInput,
    );
  });
  group.bench_function("owned_receiver/string_result", |bencher| {
    bencher.iter_batched(
      || PathBuf::from(owned_case.path),
      |input| black_box(black_box(input).into_slash()),
      BatchSize::SmallInput,
    );
  });
  group.finish();

  #[cfg(target_family = "windows")]
  {
    let mut group = criterion.benchmark_group("slash/windows_separator_branches");
    for case in WINDOWS_SLASH_CASES {
      group.throughput(Throughput::Bytes(case.path.len() as u64));
      group.bench_with_input(
        BenchmarkId::new("borrowed_receiver/strict_natural_result", case.name),
        case,
        |bencher, case| {
          bencher.iter(|| {
            let path = Path::new(black_box(case.path));
            black_box(path.to_slash())
          });
        },
      );
      group.bench_with_input(
        BenchmarkId::new("borrowed_receiver/lossy_natural_result", case.name),
        case,
        |bencher, case| {
          bencher.iter(|| {
            let path = Path::new(black_box(case.path));
            black_box(path.to_slash_lossy())
          });
        },
      );
    }
    group.finish();
  }

  #[cfg(any(unix, windows))]
  {
    let invalid = invalid_unicode_path();
    let mut group = criterion.benchmark_group("slash/invalid_unicode");
    group.throughput(Throughput::Bytes(invalid.as_os_str().len() as u64));
    group.bench_function("borrowed_receiver/fallible_result", |bencher| {
      bencher.iter(|| black_box(black_box(invalid.as_path()).try_to_slash()));
    });
    group.bench_function("borrowed_receiver/lossy_result", |bencher| {
      bencher.iter(|| black_box(black_box(invalid.as_path()).to_slash_lossy()));
    });
    group.finish();
  }
}

criterion_group!(benches, bench_to_slash);
criterion_main!(benches);
