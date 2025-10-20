use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod fixtures;

use fixtures::FIXTURES;

fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("as_path_str", |b| {
    b.iter(|| {
      for fixture in FIXTURES {
        // Benchmark converting &str to Path using as_path
        let path = black_box(fixture.as_path());
        black_box(path);
      }
    })
  });

  c.bench_function("as_path_string", |b| {
    let string_fixtures: Vec<String> = FIXTURES.iter().map(|s| s.to_string()).collect();
    b.iter(|| {
      for fixture in &string_fixtures {
        // Benchmark converting String to Path using as_path
        let path = black_box(fixture.as_path());
        black_box(path);
      }
    })
  });

  c.bench_function("as_path_vs_path_new", |b| {
    b.iter(|| {
      for fixture in FIXTURES {
        // Compare as_path() with Path::new() for baseline
        let path1 = black_box(fixture.as_path());
        let path2 = black_box(Path::new(fixture));
        black_box((path1, path2));
      }
    })
  });

  c.bench_function("as_path_chaining", |b| {
    b.iter(|| {
      for fixture in FIXTURES {
        // Benchmark as_path followed by other operations
        let normalized = black_box(fixture.as_path().normalize());
        black_box(normalized);
      }
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
