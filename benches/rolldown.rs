use std::borrow::Cow;
use std::hint::black_box;
use std::path::Path;

use arcstr::ArcStr;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use sugar_path::SugarPath;

mod support;

use support::workloads::{RELATIVE_CASES, ROLLDOWN_ROOT, RelativeCase};

#[cfg(not(target_family = "windows"))]
const DIRTY_JOINED_SPECIFIER: &str = "./crates/rolldown/src/../src/module_loader/module_task.rs";
#[cfg(target_family = "windows")]
const DIRTY_JOINED_SPECIFIER: &str = r".\crates\rolldown\src\..\src\module_loader\module_task.rs";
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

fn bench_relative_result(criterion: &mut Criterion, descendant: RelativeCase) {
  let input_bytes = (descendant.target.len() + descendant.base.len()) as u64;
  let mut group = criterion.benchmark_group("rolldown/relative_result");
  group.throughput(Throughput::Bytes(input_bytes));
  // v2's natural result is already an owned PathBuf. The paired rows are
  // intentionally identical here so a later API can retain these IDs while
  // separating its natural result from a requested PathBuf output.

  group.bench_function("path_receiver/natural_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      black_box(target.relative(base))
    });
  });
  group.bench_function("path_receiver/pathbuf_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      black_box(target.relative(base))
    });
  });
  group.bench_function("str_receiver/natural_result", |bencher| {
    bencher.iter(|| {
      let target = black_box(descendant.target);
      let base = black_box(descendant.base);
      black_box(target.relative(base))
    });
  });
  group.bench_function("str_receiver/pathbuf_result", |bencher| {
    bencher.iter(|| {
      let target = black_box(descendant.target);
      let base = black_box(descendant.base);
      black_box(target.relative(base))
    });
  });
  group.finish();
}

fn bench_relative_string_result(
  criterion: &mut Criterion,
  descendant: RelativeCase,
  upward: RelativeCase,
) {
  let mut group = criterion.benchmark_group("rolldown/relative_string_result");
  group.throughput(Throughput::Bytes((descendant.target.len() + descendant.base.len()) as u64));

  group.bench_function("descendant/borrowed_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      let relative = target.relative(base);
      black_box(relative.to_slash_lossy().into_owned())
    });
  });
  group.bench_function("descendant/owned_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      let relative = target.relative(base);
      black_box(relative.to_slash_lossy().into_owned())
    });
  });
  group.bench_function("descendant/borrowed_receiver/normalized_string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      let relative = target.relative(base);
      let normalized = relative.normalize();
      black_box(normalized.to_slash_lossy().into_owned())
    });
  });
  group.bench_function("descendant/owned_receiver/normalized_string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      let relative = target.relative(base);
      let normalized = relative.normalize();
      black_box(normalized.to_slash_lossy().into_owned())
    });
  });

  group.throughput(Throughput::Bytes((upward.target.len() + upward.base.len()) as u64));
  group.bench_function("upward/borrowed_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(upward.target));
      let base = Path::new(black_box(upward.base));
      let relative = target.relative(base);
      black_box(relative.to_slash_lossy().into_owned())
    });
  });
  group.bench_function("upward/owned_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(upward.target));
      let base = Path::new(black_box(upward.base));
      let relative = target.relative(base);
      black_box(relative.to_slash_lossy().into_owned())
    });
  });
  group.bench_function("upward/borrowed_receiver/normalized_string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(upward.target));
      let base = Path::new(black_box(upward.base));
      let relative = target.relative(base);
      let normalized = relative.normalize();
      black_box(normalized.to_slash_lossy().into_owned())
    });
  });
  group.bench_function("upward/owned_receiver/normalized_string_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(upward.target));
      let base = Path::new(black_box(upward.base));
      let relative = target.relative(base);
      let normalized = relative.normalize();
      black_box(normalized.to_slash_lossy().into_owned())
    });
  });
  group.finish();
}

fn bench_relative_arcstr_result(
  criterion: &mut Criterion,
  descendant: RelativeCase,
  upward: RelativeCase,
) {
  let mut group = criterion.benchmark_group("rolldown/relative_arcstr_result");
  group.throughput(Throughput::Bytes((descendant.target.len() + descendant.base.len()) as u64));

  group.bench_function("descendant/borrowed_receiver/natural_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      let relative = target.relative(base);
      let slash = relative.to_slash().expect("benchmark paths are valid UTF-8");
      black_box(ArcStr::from(slash))
    });
  });
  group.bench_function("descendant/borrowed_receiver/string_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      let relative = target.relative(base);
      let slash = relative.to_slash().expect("benchmark paths are valid UTF-8").into_owned();
      black_box(ArcStr::from(slash))
    });
  });
  // The v2 API cannot consume the relative PathBuf during slash conversion.
  // Keep the same final String-to-ArcStr operation under the owned-receiver ID
  // so the later API can measure storage reuse without changing the output.
  group.bench_function("descendant/owned_receiver/string_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      let relative = target.relative(base);
      let slash = relative.to_slash().expect("benchmark paths are valid UTF-8").into_owned();
      black_box(ArcStr::from(slash))
    });
  });

  group.throughput(Throughput::Bytes((upward.target.len() + upward.base.len()) as u64));
  group.bench_function("upward/borrowed_receiver/natural_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(upward.target));
      let base = Path::new(black_box(upward.base));
      let relative = target.relative(base);
      let slash = relative.to_slash().expect("benchmark paths are valid UTF-8");
      black_box(ArcStr::from(slash))
    });
  });
  group.bench_function("upward/borrowed_receiver/string_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(upward.target));
      let base = Path::new(black_box(upward.base));
      let relative = target.relative(base);
      let slash = relative.to_slash().expect("benchmark paths are valid UTF-8").into_owned();
      black_box(ArcStr::from(slash))
    });
  });
  group.bench_function("upward/owned_receiver/string_slash", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(upward.target));
      let base = Path::new(black_box(upward.base));
      let relative = target.relative(base);
      let slash = relative.to_slash().expect("benchmark paths are valid UTF-8").into_owned();
      black_box(ArcStr::from(slash))
    });
  });
  group.finish();
}

fn bench_side_effects_relative(
  criterion: &mut Criterion,
  descendant: RelativeCase,
  upward: RelativeCase,
) {
  let mut group = criterion.benchmark_group("rolldown/side_effects_relative");
  group.throughput(Throughput::Bytes((descendant.target.len() + descendant.base.len()) as u64));

  group.bench_function("descendant/borrowed_receiver/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      let relative = target.relative(base);
      black_box(relative.to_str().expect("benchmark paths are valid UTF-8"));
    });
  });
  group.bench_function("descendant/owned_receiver/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      let relative = target.relative(base);
      black_box(relative.to_str().expect("benchmark paths are valid UTF-8"));
    });
  });
  group.bench_function("descendant/strip_prefix_fallback/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(descendant.target));
      let base = Path::new(black_box(descendant.base));
      let relative = target
        .strip_prefix(base)
        .map(Cow::Borrowed)
        .unwrap_or_else(|_| Cow::Owned(target.relative(base)));
      black_box(relative.to_str().expect("benchmark paths are valid UTF-8"));
    });
  });

  group.throughput(Throughput::Bytes((upward.target.len() + upward.base.len()) as u64));
  group.bench_function("upward/borrowed_receiver/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(upward.target));
      let base = Path::new(black_box(upward.base));
      let relative = target.relative(base);
      black_box(relative.to_str().expect("benchmark paths are valid UTF-8"));
    });
  });
  group.bench_function("upward/owned_receiver/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(upward.target));
      let base = Path::new(black_box(upward.base));
      let relative = target.relative(base);
      black_box(relative.to_str().expect("benchmark paths are valid UTF-8"));
    });
  });
  group.bench_function("upward/strip_prefix_fallback/text_result", |bencher| {
    bencher.iter(|| {
      let target = Path::new(black_box(upward.target));
      let base = Path::new(black_box(upward.base));
      let relative = target
        .strip_prefix(base)
        .map(Cow::Borrowed)
        .unwrap_or_else(|_| Cow::Owned(target.relative(base)));
      black_box(relative.to_str().expect("benchmark paths are valid UTF-8"));
    });
  });
  group.finish();
}

fn bench_join_result(criterion: &mut Criterion, group_name: &str, specifier: &'static str) {
  let mut group = criterion.benchmark_group(group_name);
  group.throughput(Throughput::Bytes((ROLLDOWN_ROOT.len() + specifier.len()) as u64));

  group.bench_function("borrowed_receiver/pathbuf_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(specifier));
      black_box(joined.normalize().into_owned())
    });
  });
  group.bench_function("owned_receiver/pathbuf_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(specifier));
      black_box(joined.normalize().into_owned())
    });
  });
  group.bench_function("borrowed_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(specifier));
      let normalized = joined.normalize();
      black_box(normalized.to_slash_lossy().into_owned())
    });
  });
  group.bench_function("owned_receiver/string_result", |bencher| {
    bencher.iter(|| {
      let base = Path::new(black_box(ROLLDOWN_ROOT));
      let joined = base.join(black_box(specifier));
      let normalized = joined.normalize();
      black_box(normalized.to_slash_lossy().into_owned())
    });
  });
  group.finish();
}

fn bench_rolldown_pipelines(criterion: &mut Criterion) {
  let descendant = relative_case("module_to_cwd");
  let upward = relative_case("deep_siblings");
  let side_effects_upward = relative_case("different_subtrees");

  bench_relative_result(criterion, descendant);
  bench_relative_string_result(criterion, descendant, upward);
  bench_relative_arcstr_result(criterion, descendant, upward);
  bench_side_effects_relative(criterion, descendant, side_effects_upward);
  bench_join_result(criterion, "rolldown/join_dirty_result", DIRTY_JOINED_SPECIFIER);
  bench_join_result(criterion, "rolldown/join_clean_result", CLEAN_JOINED_SPECIFIER);
}

criterion_group!(benches, bench_rolldown_pipelines);
criterion_main!(benches);
