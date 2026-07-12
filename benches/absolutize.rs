use std::hint::black_box;
use std::path::{Path, PathBuf};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod support;

use support::workloads::{ROLLDOWN_PATHS, ROLLDOWN_ROOT};

#[cfg(not(target_family = "windows"))]
const RELATIVE_MODULE: &str = "crates/rolldown/src/module_loader/external_module_task.rs";
#[cfg(target_family = "windows")]
const RELATIVE_MODULE: &str = r"crates\rolldown\src\module_loader\external_module_task.rs";

#[cfg(not(target_family = "windows"))]
const DIRTY_RELATIVE_MODULE: &str = "crates/rolldown/src/module_loader/../bundle/bundle.rs";
#[cfg(target_family = "windows")]
const DIRTY_RELATIVE_MODULE: &str = r"crates\rolldown\src\module_loader\..\bundle\bundle.rs";

fn bench_absolutize(criterion: &mut Criterion) {
  let clean_absolute = ROLLDOWN_PATHS[2].path;

  let mut group = criterion.benchmark_group("absolutize/current_dir");
  group.throughput(Throughput::Bytes(clean_absolute.len() as u64));
  group.bench_function("clean_absolute", |bencher| {
    bencher.iter(|| {
      let input = Path::new(black_box(clean_absolute));
      black_box(input.absolutize())
    });
  });
  group.throughput(Throughput::Bytes(RELATIVE_MODULE.len() as u64));
  group.bench_function("relative", |bencher| {
    bencher.iter(|| {
      let input = Path::new(black_box(RELATIVE_MODULE));
      black_box(input.absolutize())
    });
  });
  group.finish();

  let mut group = criterion.benchmark_group("absolutize_with/borrowed_cwd");
  group.throughput(Throughput::Bytes(clean_absolute.len() as u64));
  group.bench_function("clean_absolute", |bencher| {
    bencher.iter(|| {
      let input = Path::new(black_box(clean_absolute));
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      black_box(input.absolutize_with(base))
    });
  });
  for (name, path) in
    [("relative_clean", RELATIVE_MODULE), ("relative_dirty", DIRTY_RELATIVE_MODULE)]
  {
    group.throughput(Throughput::Bytes((path.len() + ROLLDOWN_ROOT.len()) as u64));
    group.bench_function(name, |bencher| {
      bencher.iter(|| {
        let input = Path::new(black_box(path));
        let base = Path::new(black_box(ROLLDOWN_ROOT));
        black_box(input.absolutize_with(base))
      });
    });
  }
  group.finish();

  let mut group = criterion.benchmark_group("absolutize_with/owned_cwd");
  group.throughput(Throughput::Bytes((RELATIVE_MODULE.len() + ROLLDOWN_ROOT.len()) as u64));
  group.bench_function("relative_clean", |bencher| {
    bencher.iter_with_setup_wrapper(|runner| {
      let base = PathBuf::from(black_box(ROLLDOWN_ROOT));
      runner.run(|| {
        let input = Path::new(black_box(RELATIVE_MODULE));
        let output = input.absolutize_with(base);
        drop(black_box(output));
      });
    });
  });
  group.finish();
}

criterion_group!(benches, bench_absolutize);
criterion_main!(benches);
