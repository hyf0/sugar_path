use std::hint::black_box;
use std::path::{Path, PathBuf};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod support;

#[cfg(any(unix, windows))]
use support::workloads::invalid_unicode_path;
use support::workloads::{
  CANONICAL_LEADING_PARENTS, CURRENT_DIRECTORY_CASES, DIRTY_PATHS, LEADING_PARENT_SCAN_CASES,
  PathCase, ROLLDOWN_PATHS,
};

fn bench_path_cases(criterion: &mut Criterion, group_name: &str, cases: &[PathCase]) {
  let mut group = criterion.benchmark_group(group_name);
  for case in cases {
    group.throughput(Throughput::Bytes(case.path.len() as u64));
    group.bench_with_input(BenchmarkId::from_parameter(case.name), case, |bencher, case| {
      bencher.iter(|| {
        let input = Path::new(black_box(case.path));
        black_box(input.normalize())
      });
    });
  }
  group.finish();
}

fn bench_normalize(criterion: &mut Criterion) {
  bench_path_cases(criterion, "normalize/clean_rolldown", ROLLDOWN_PATHS);
  bench_path_cases(criterion, "normalize/needs_work", DIRTY_PATHS);
  bench_path_cases(criterion, "normalize/canonical_leading_parents", &[CANONICAL_LEADING_PARENTS]);
  bench_path_cases(criterion, "normalize/leading_parent_prescan", LEADING_PARENT_SCAN_CASES);
  bench_path_cases(criterion, "normalize/current_directory_spellings", CURRENT_DIRECTORY_CASES);

  let total_bytes = ROLLDOWN_PATHS.iter().map(|case| case.path.len() as u64).sum();
  let mut group = criterion.benchmark_group("normalize/rolldown_corpus");
  group.throughput(Throughput::Bytes(total_bytes));
  group.bench_function("clean", |bencher| {
    bencher.iter(|| {
      for case in black_box(ROLLDOWN_PATHS) {
        let input = Path::new(black_box(case.path));
        black_box(input.normalize());
      }
    });
  });
  group.finish();

  #[cfg(any(unix, windows))]
  {
    let invalid = invalid_unicode_path();
    let mut group = criterion.benchmark_group("normalize/invalid_encoding");
    group.throughput(Throughput::Bytes(invalid.as_os_str().len() as u64));
    group.bench_function("lexically_clean", |bencher| {
      bencher.iter(|| black_box(black_box(invalid.as_path()).normalize()));
    });
    group.finish();
  }

  for case in [&ROLLDOWN_PATHS[2], &DIRTY_PATHS[1]] {
    let mut group = criterion.benchmark_group(format!("normalize/owned_input/{}", case.name));
    group.throughput(Throughput::Bytes(case.path.len() as u64));
    group.bench_function("borrowed_receiver/pathbuf_result", |bencher| {
      bencher.iter_batched(
        || PathBuf::from(case.path),
        |input| black_box(black_box(input.as_path()).normalize().into_owned()),
        BatchSize::SmallInput,
      );
    });
    // The v2 API has no consuming normalization method. Keeping the same
    // baseline operation under this output-oriented ID lets v3 measure
    // whether consuming the owned input improves the same PathBuf result.
    group.bench_function("owned_receiver/pathbuf_result", |bencher| {
      bencher.iter_batched(
        || PathBuf::from(case.path),
        |input| black_box(black_box(input.as_path()).normalize().into_owned()),
        BatchSize::SmallInput,
      );
    });
    group.finish();
  }
}

criterion_group!(benches, bench_normalize);
criterion_main!(benches);
