use std::borrow::Cow;
use std::hint::black_box;
use std::path::Path;

use arcstr::ArcStr;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use sugar_path::{SugarPath, SugarPathBuf};

mod support;

use support::workloads::{RELATIVE_CASES, ROLLDOWN_ROOT, RelativeCase};

#[cfg(not(target_family = "windows"))]
const JOINED_SPECIFIER: &str = "./crates/rolldown/src/../src/module_loader/module_task.rs";
#[cfg(target_family = "windows")]
const JOINED_SPECIFIER: &str = r".\crates\rolldown\src\..\src\module_loader\module_task.rs";
#[cfg(not(target_family = "windows"))]
const CLEAN_JOINED_SPECIFIER: &str = "crates/rolldown/src/module_loader/module_task.rs";
#[cfg(target_family = "windows")]
const CLEAN_JOINED_SPECIFIER: &str = r"crates\rolldown\src\module_loader\module_task.rs";

fn relative_case(name: &str) -> RelativeCase {
  RELATIVE_CASES
    .iter()
    .copied()
    .find(|case| case.name == name)
    .unwrap_or_else(|| panic!("missing benchmark case {name}"))
}

fn bench_rolldown_pipelines(criterion: &mut Criterion) {
  let case = relative_case("module_to_cwd");
  let leading_parent_case = relative_case("deep_siblings");
  let input_bytes = (case.target.len() + case.base.len()) as u64;
  let mut group = criterion.benchmark_group("rolldown/relative_result");
  group.throughput(Throughput::Bytes(input_bytes));

  group.bench_function("path_receiver/natural_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(case.target));
      let base = Path::new(black_box(case.base));
      black_box(target.relative(base))
    });
  });

  group.bench_function("str_receiver/natural_result", |bencher| {
    bencher.iter(|| {
      let target = black_box(case.target);
      let base = black_box(case.base);
      black_box(target.relative(base))
    });
  });

  group.bench_function("path_receiver/pathbuf_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(case.target));
      let base = Path::new(black_box(case.base));
      black_box(target.relative(base).into_owned())
    });
  });

  group.bench_function("str_receiver/pathbuf_result", |bencher| {
    bencher.iter(|| {
      let target = black_box(case.target);
      let base = black_box(case.base);
      black_box(target.relative(base).into_owned())
    });
  });
  group.finish();

  let mut group = criterion.benchmark_group("rolldown/relative_string_result");
  group.throughput(Throughput::Bytes(input_bytes));

  group.bench_function("descendant/borrowed_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(case.target));
      let base = Path::new(black_box(case.base));
      let relative = target.relative(base);
      black_box(relative.to_slash().into_owned())
    });
  });

  group.bench_function("descendant/owned_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(case.target));
      let base = Path::new(black_box(case.base));
      black_box(target.relative(base).into_owned().into_slash())
    });
  });

  group.bench_function("descendant/borrowed_receiver/normalized_string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(case.target));
      let base = Path::new(black_box(case.base));
      let relative = target.relative(base);
      let normalized = relative.normalize();
      black_box(normalized.to_slash().into_owned())
    });
  });

  group.bench_function("descendant/owned_receiver/normalized_string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(case.target));
      let base = Path::new(black_box(case.base));
      black_box(target.relative(base).into_owned().into_normalized().into_slash())
    });
  });

  group.throughput(Throughput::Bytes(
    (leading_parent_case.target.len() + leading_parent_case.base.len()) as u64,
  ));
  group.bench_function("upward/borrowed_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(leading_parent_case.target));
      let base = Path::new(black_box(leading_parent_case.base));
      let relative = target.relative(base);
      black_box(relative.to_slash().into_owned())
    });
  });

  group.bench_function("upward/owned_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(leading_parent_case.target));
      let base = Path::new(black_box(leading_parent_case.base));
      black_box(target.relative(base).into_owned().into_slash())
    });
  });

  group.bench_function("upward/borrowed_receiver/normalized_string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(leading_parent_case.target));
      let base = Path::new(black_box(leading_parent_case.base));
      let relative = target.relative(base);
      let normalized = relative.normalize();
      black_box(normalized.to_slash().into_owned())
    });
  });

  group.bench_function("upward/owned_receiver/normalized_string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(leading_parent_case.target));
      let base = Path::new(black_box(leading_parent_case.base));
      black_box(target.relative(base).into_owned().into_normalized().into_slash())
    });
  });

  group.finish();

  let mut group = criterion.benchmark_group("rolldown/relative_arcstr_result");
  group.throughput(Throughput::Bytes(input_bytes));
  group.bench_function("descendant/borrowed_receiver/natural_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(case.target));
      let base = Path::new(black_box(case.base));
      let relative = target.relative(base);
      black_box(ArcStr::from(relative.to_slash()))
    });
  });
  group.bench_function("descendant/borrowed_receiver/string_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(case.target));
      let base = Path::new(black_box(case.base));
      let relative = target.relative(base);
      black_box(ArcStr::from(relative.to_slash().into_owned()))
    });
  });
  group.bench_function("descendant/owned_receiver/string_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(case.target));
      let base = Path::new(black_box(case.base));
      black_box(ArcStr::from(target.relative(base).into_owned().into_slash()))
    });
  });

  group.throughput(Throughput::Bytes(
    (leading_parent_case.target.len() + leading_parent_case.base.len()) as u64,
  ));
  group.bench_function("upward/borrowed_receiver/natural_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(leading_parent_case.target));
      let base = Path::new(black_box(leading_parent_case.base));
      let relative = target.relative(base);
      black_box(ArcStr::from(relative.to_slash()))
    });
  });
  group.bench_function("upward/borrowed_receiver/string_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(leading_parent_case.target));
      let base = Path::new(black_box(leading_parent_case.base));
      let relative = target.relative(base);
      black_box(ArcStr::from(relative.to_slash().into_owned()))
    });
  });
  group.bench_function("upward/owned_receiver/string_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(leading_parent_case.target));
      let base = Path::new(black_box(leading_parent_case.base));
      black_box(ArcStr::from(target.relative(base).into_owned().into_slash()))
    });
  });
  group.finish();

  let side_effects_hit = relative_case("module_to_cwd");
  let side_effects_miss = relative_case("different_subtrees");
  let mut group = criterion.benchmark_group("rolldown/side_effects_relative");
  group.throughput(Throughput::Bytes(
    (side_effects_hit.target.len() + side_effects_hit.base.len()) as u64,
  ));
  group.bench_function("descendant/borrowed_receiver/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(side_effects_hit.target));
      let base = Path::new(black_box(side_effects_hit.base));
      let relative = target.relative(base);
      black_box(relative.to_str().expect("Rolldown paths are valid UTF-8"));
    });
  });
  group.bench_function("descendant/owned_receiver/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(side_effects_hit.target));
      let base = Path::new(black_box(side_effects_hit.base));
      let relative = target.relative(base).into_owned();
      black_box(relative.to_str().expect("Rolldown paths are valid UTF-8"));
    });
  });
  group.bench_function("descendant/strip_prefix_fallback/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(side_effects_hit.target));
      let base = Path::new(black_box(side_effects_hit.base));
      let relative =
        target.strip_prefix(base).map(Cow::Borrowed).unwrap_or_else(|_| target.relative(base));
      black_box(relative.to_str().expect("Rolldown paths are valid UTF-8"));
    });
  });

  group.throughput(Throughput::Bytes(
    (side_effects_miss.target.len() + side_effects_miss.base.len()) as u64,
  ));
  group.bench_function("upward/borrowed_receiver/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(side_effects_miss.target));
      let base = Path::new(black_box(side_effects_miss.base));
      let relative = target.relative(base);
      black_box(relative.to_str().expect("Rolldown paths are valid UTF-8"));
    });
  });
  group.bench_function("upward/owned_receiver/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(side_effects_miss.target));
      let base = Path::new(black_box(side_effects_miss.base));
      let relative = target.relative(base).into_owned();
      black_box(relative.to_str().expect("Rolldown paths are valid UTF-8"));
    });
  });
  group.bench_function("upward/strip_prefix_fallback/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(side_effects_miss.target));
      let base = Path::new(black_box(side_effects_miss.base));
      let relative =
        target.strip_prefix(base).map(Cow::Borrowed).unwrap_or_else(|_| target.relative(base));
      black_box(relative.to_str().expect("Rolldown paths are valid UTF-8"));
    });
  });
  group.finish();

  let mut group = criterion.benchmark_group("rolldown/join_dirty_result");
  group.throughput(Throughput::Bytes((ROLLDOWN_ROOT.len() + JOINED_SPECIFIER.len()) as u64));
  group.bench_function("borrowed_receiver/pathbuf_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(JOINED_SPECIFIER));
      black_box(joined.normalize().into_owned())
    });
  });
  group.bench_function("owned_receiver/pathbuf_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(JOINED_SPECIFIER));
      black_box(joined.into_normalized())
    });
  });
  group.bench_function("borrowed_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(JOINED_SPECIFIER));
      let normalized = joined.normalize();
      black_box(normalized.to_slash().into_owned())
    });
  });
  group.bench_function("owned_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(JOINED_SPECIFIER));
      black_box(joined.into_normalized().into_slash())
    });
  });
  group.finish();

  let mut group = criterion.benchmark_group("rolldown/join_clean_result");
  group.throughput(Throughput::Bytes((ROLLDOWN_ROOT.len() + CLEAN_JOINED_SPECIFIER.len()) as u64));
  group.bench_function("borrowed_receiver/pathbuf_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(CLEAN_JOINED_SPECIFIER));
      black_box(joined.normalize().into_owned())
    });
  });
  group.bench_function("owned_receiver/pathbuf_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(CLEAN_JOINED_SPECIFIER));
      black_box(joined.into_normalized())
    });
  });
  group.bench_function("borrowed_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(CLEAN_JOINED_SPECIFIER));
      black_box(joined.normalize().to_slash().into_owned())
    });
  });
  group.bench_function("owned_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(CLEAN_JOINED_SPECIFIER));
      black_box(joined.into_normalized().into_slash())
    });
  });
  group.finish();
}

criterion_group!(benches, bench_rolldown_pipelines);
criterion_main!(benches);
