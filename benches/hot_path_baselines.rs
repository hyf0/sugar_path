use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod support;

#[cfg(all(target_family = "unix", not(target_os = "cygwin")))]
use support::workloads::RELATIVE_CASES;
#[cfg(any(unix, windows))]
use support::workloads::{ROLLDOWN_ROOT, non_utf8_path};

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
    let non_utf8 = non_utf8_path();
    let non_utf8_name = non_utf8.file_name().expect("non-UTF-8 fixture has a file name").to_owned();
    let mut dirty_before_non_utf8 = non_utf8.clone();
    dirty_before_non_utf8.pop();
    dirty_before_non_utf8.push(".");
    dirty_before_non_utf8.push(&non_utf8_name);
    let mut non_utf8_before_dirty_late = non_utf8.clone();
    non_utf8_before_dirty_late.push("late");
    non_utf8_before_dirty_late.push(".");
    non_utf8_before_dirty_late.push("file.js");
    let mut group = criterion.benchmark_group("normalize/non_utf8");
    group.throughput(Throughput::Bytes(dirty_before_non_utf8.as_os_str().len() as u64));
    group.bench_function("dirty_before_non_utf8", |bencher| {
      bencher.iter(|| black_box(black_box(dirty_before_non_utf8.as_path()).normalize()));
    });
    group.throughput(Throughput::Bytes(non_utf8_before_dirty_late.as_os_str().len() as u64));
    group.bench_function("non_utf8_before_dirty_late", |bencher| {
      bencher.iter(|| black_box(black_box(non_utf8_before_dirty_late.as_path()).normalize()));
    });
    group.finish();

    let base = Path::new(ROLLDOWN_ROOT);
    let mut group = criterion.benchmark_group("relative/slow_path");
    group
      .throughput(Throughput::Bytes((non_utf8.as_os_str().len() + base.as_os_str().len()) as u64));
    group.bench_function("non_utf8_absolute_target", |bencher| {
      bencher.iter(|| black_box(non_utf8.as_path()).relative(black_box(base)));
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
