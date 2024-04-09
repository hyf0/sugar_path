use std::path::Path;

use criterion::{criterion_group, criterion_main, Criterion};
use sugar_path::SugarPath;



fn absolutize() {
  // "./hello".absolutize();
  "/hello".absolutize();
}

fn absolutize_with(cwd: &Path) {
  // "./hello".absolutize_with(cwd);
  "/hello".absolutize_with(cwd);
}

fn criterion_benchmark(c: &mut Criterion) {
  let cwd = std::env::current_dir().unwrap();
    c.bench_function("absolutize", |b| b.iter(absolutize));
    c.bench_function("absolutize_with", |b| b.iter(|| absolutize_with(&cwd)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);