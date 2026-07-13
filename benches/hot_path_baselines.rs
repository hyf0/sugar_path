use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod support;

#[cfg(all(target_family = "unix", not(target_os = "cygwin")))]
use support::workloads::RELATIVE_CASES;
#[cfg(any(unix, windows))]
use support::workloads::{ROLLDOWN_ROOT, invalid_unicode_path};

fn bench_hot_path_baselines(criterion: &mut Criterion) {
  #[cfg(not(target_family = "windows"))]
  let (dirty_early, dirty_late) = (
    "/./workspace/rolldown/crates/rolldown/src/module_loader/module_task.rs",
    "/workspace/rolldown/crates/rolldown/src/module_loader/./module_task.rs",
  );
  #[cfg(target_family = "windows")]
  let (dirty_early, dirty_late) = (
    r"C:\.\workspace\rolldown\crates\rolldown\src\module_loader\module_task.rs",
    r"C:\workspace\rolldown\crates\rolldown\src\module_loader\.\module_task.rs",
  );
  let mut group = criterion.benchmark_group("normalize/classifier_position/valid");
  for (name, path) in [("dirty_early", dirty_early), ("dirty_late", dirty_late)] {
    group.throughput(Throughput::Bytes(path.len() as u64));
    group.bench_function(name, |bencher| {
      bencher.iter(|| black_box(Path::new(black_box(path)).normalize()));
    });
  }
  group.finish();

  #[cfg(any(unix, windows))]
  {
    let invalid = invalid_unicode_path();
    let invalid_name = invalid.file_name().expect("invalid fixture has a file name").to_owned();
    let mut dirty_before_invalid = invalid.clone();
    dirty_before_invalid.pop();
    dirty_before_invalid.push(".");
    dirty_before_invalid.push(&invalid_name);
    let mut invalid_before_dirty_late = invalid.clone();
    invalid_before_dirty_late.push("late");
    invalid_before_dirty_late.push(".");
    invalid_before_dirty_late.push("file.js");
    let mut group = criterion.benchmark_group("normalize/invalid_encoding");
    group.throughput(Throughput::Bytes(dirty_before_invalid.as_os_str().len() as u64));
    group.bench_function("dirty_before_invalid", |bencher| {
      bencher.iter(|| black_box(black_box(dirty_before_invalid.as_path()).normalize()));
    });
    group.throughput(Throughput::Bytes(invalid_before_dirty_late.as_os_str().len() as u64));
    group.bench_function("invalid_before_dirty_late", |bencher| {
      bencher.iter(|| black_box(black_box(invalid_before_dirty_late.as_path()).normalize()));
    });
    group.finish();

    let base = Path::new(ROLLDOWN_ROOT);
    let mut group = criterion.benchmark_group("relative/slow_path");
    group
      .throughput(Throughput::Bytes((invalid.as_os_str().len() + base.as_os_str().len()) as u64));
    group.bench_function("invalid_encoding_absolute", |bencher| {
      bencher.iter(|| black_box(invalid.as_path()).relative(black_box(base)));
    });
    group.finish();
  }

  #[cfg(all(target_family = "unix", not(target_os = "cygwin")))]
  {
    let case = &RELATIVE_CASES[0];
    let mut group = criterion.benchmark_group("relative/string_receiver/natural_result");
    group.throughput(Throughput::Bytes((case.target.len() + case.base.len()) as u64));
    group.bench_function(case.name, |bencher| {
      bencher.iter(|| black_box(black_box(case.target).relative(black_box(case.base))));
    });
    group.finish();
  }
}

criterion_group!(benches, bench_hot_path_baselines);
criterion_main!(benches);
