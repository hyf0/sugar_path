use std::alloc::{GlobalAlloc, Layout};
use std::borrow::Cow;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use arcstr::ArcStr;
use mimalloc_safe::MiMalloc;
use sugar_path::{SugarPath, SugarPathBuf};

const MEASUREMENTS: usize = 7;
const TARGET_ENVIRONMENT: &str = if cfg!(target_env = "gnu") {
  "gnu"
} else if cfg!(target_env = "msvc") {
  "msvc"
} else if cfg!(target_env = "musl") {
  "musl"
} else {
  "other"
};

struct CountingAllocator {
  inner: MiMalloc,
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator { inner: MiMalloc };

static TRACKING: AtomicBool = AtomicBool::new(false);
static ALLOC_CALLS: AtomicUsize = AtomicUsize::new(0);
static ALLOC_ZEROED_CALLS: AtomicUsize = AtomicUsize::new(0);
static REALLOC_CALLS: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);
static REALLOC_NEW_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let ptr = unsafe { self.inner.alloc(layout) };
    if !ptr.is_null() && TRACKING.load(Ordering::Relaxed) {
      ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
      ALLOC_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
    }
    ptr
  }

  unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
    let ptr = unsafe { self.inner.alloc_zeroed(layout) };
    if !ptr.is_null() && TRACKING.load(Ordering::Relaxed) {
      ALLOC_ZEROED_CALLS.fetch_add(1, Ordering::Relaxed);
      ALLOC_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
    }
    ptr
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe { self.inner.dealloc(ptr, layout) };
  }

  unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    let new_ptr = unsafe { self.inner.realloc(ptr, layout, new_size) };
    if !new_ptr.is_null() && TRACKING.load(Ordering::Relaxed) {
      REALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
      REALLOC_NEW_BYTES.fetch_add(new_size, Ordering::Relaxed);
    }
    new_ptr
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AllocationStats {
  alloc_calls: usize,
  alloc_zeroed_calls: usize,
  realloc_calls: usize,
  alloc_bytes: usize,
  realloc_new_bytes: usize,
}

impl AllocationStats {
  const ZERO: Self = Self {
    alloc_calls: 0,
    alloc_zeroed_calls: 0,
    realloc_calls: 0,
    alloc_bytes: 0,
    realloc_new_bytes: 0,
  };

  fn load() -> Self {
    Self {
      alloc_calls: ALLOC_CALLS.load(Ordering::Relaxed),
      alloc_zeroed_calls: ALLOC_ZEROED_CALLS.load(Ordering::Relaxed),
      realloc_calls: REALLOC_CALLS.load(Ordering::Relaxed),
      alloc_bytes: ALLOC_BYTES.load(Ordering::Relaxed),
      realloc_new_bytes: REALLOC_NEW_BYTES.load(Ordering::Relaxed),
    }
  }
}

struct TrackingGuard;

impl TrackingGuard {
  fn begin() -> Self {
    assert!(!TRACKING.swap(true, Ordering::SeqCst), "allocation tracking regions must not nest");
    Self
  }
}

impl Drop for TrackingGuard {
  fn drop(&mut self) {
    TRACKING.store(false, Ordering::SeqCst);
  }
}

#[derive(Clone, Copy)]
struct Scenario {
  name: &'static str,
  run: fn(RunMode) -> AllocationStats,
}

#[derive(Clone, Copy)]
struct Measurement {
  name: &'static str,
  stats: AllocationStats,
}

#[derive(Clone, Copy)]
enum RunMode {
  Warm,
  Measure,
}

#[derive(Clone, Copy)]
struct CwdShape {
  encoded_bytes: usize,
  components: usize,
}

struct MeasurementCwd {
  previous: Option<PathBuf>,
  shape: CwdShape,
}

impl MeasurementCwd {
  fn enter() -> Result<Self, String> {
    let previous = std::env::current_dir()
      .map_err(|error| format!("failed to read the original current directory: {error}"))?;
    let requested = measurement_cwd_path();
    fs::create_dir_all(&requested).map_err(|error| {
      format!("failed to create the deterministic measurement directory: {error}")
    })?;
    std::env::set_current_dir(&requested).map_err(|error| {
      format!("failed to enter the deterministic measurement directory: {error}")
    })?;
    let actual = std::env::current_dir()
      .map_err(|error| format!("failed to read the measurement current directory: {error}"))?;
    let shape =
      CwdShape { encoded_bytes: actual.as_os_str().len(), components: actual.components().count() };
    Ok(Self { previous: Some(previous), shape })
  }

  fn restore(mut self) -> Result<(), String> {
    let previous = self.previous.take().expect("the original directory is restored once");
    std::env::set_current_dir(previous)
      .map_err(|error| format!("failed to restore the original current directory: {error}"))
  }
}

impl Drop for MeasurementCwd {
  fn drop(&mut self) {
    if let Some(previous) = self.previous.take() {
      let _ = std::env::set_current_dir(previous);
    }
  }
}

#[cfg(target_family = "unix")]
fn measurement_cwd_path() -> PathBuf {
  PathBuf::from("/tmp/sugar_path_track_allocations/cwd")
}

#[cfg(target_family = "windows")]
fn measurement_cwd_path() -> PathBuf {
  std::env::temp_dir().join("sugar_path_track_allocations").join("cwd")
}

#[cfg(not(any(target_family = "unix", target_family = "windows")))]
fn measurement_cwd_path() -> PathBuf {
  std::env::temp_dir().join("sugar_path_track_allocations_cwd")
}

const SCENARIOS: &[Scenario] = &[
  Scenario { name: "normalize / clean input -> normalized path", run: normalize_clean },
  Scenario { name: "normalize / dirty input -> normalized path", run: normalize_dirty },
  Scenario {
    name: "normalize / empty current-directory spelling -> normalized path",
    run: normalize_empty,
  },
  Scenario {
    name: "normalize / dot current-directory spelling -> normalized path",
    run: normalize_dot,
  },
  Scenario {
    name: "normalize / dot-separator current-directory spelling -> normalized path",
    run: normalize_dot_separator,
  },
  Scenario {
    name: "normalize / collapsing input -> normalized path",
    run: normalize_collapsing_to_current_directory,
  },
  Scenario {
    name: "normalize / noncanonical leading parents -> normalized path",
    run: normalize_leading_parents,
  },
  Scenario {
    name: "normalize / canonical leading parents -> normalized path",
    run: normalize_canonical_leading_parents,
  },
  #[cfg(any(target_family = "unix", target_family = "windows"))]
  Scenario {
    name: "normalize / clean invalid encoding -> normalized path",
    run: normalize_invalid_clean,
  },
  Scenario {
    name: "normalize / owned clean input via borrowed receiver -> PathBuf",
    run: normalize_owned_clean,
  },
  Scenario {
    name: "normalize / owned clean input via owned receiver -> PathBuf",
    run: normalize_owned_clean_consuming,
  },
  Scenario {
    name: "normalize / owned dirty input via borrowed receiver -> PathBuf",
    run: normalize_owned_dirty,
  },
  Scenario {
    name: "normalize / owned dirty input via owned receiver -> PathBuf",
    run: normalize_owned_dirty_consuming,
  },
  Scenario {
    name: "normalize / owned dot input via borrowed receiver -> PathBuf",
    run: normalize_owned_dot,
  },
  Scenario {
    name: "normalize / owned dot input via owned receiver -> PathBuf",
    run: normalize_owned_dot_consuming,
  },
  Scenario {
    name: "normalize / owned collapsing input via borrowed receiver -> PathBuf",
    run: normalize_owned_collapse,
  },
  Scenario {
    name: "normalize / owned collapsing input via owned receiver -> PathBuf",
    run: normalize_owned_collapse_consuming,
  },
  #[cfg(any(target_family = "unix", target_family = "windows"))]
  Scenario {
    name: "normalize / owned invalid input via borrowed receiver -> PathBuf",
    run: normalize_owned_invalid,
  },
  #[cfg(any(target_family = "unix", target_family = "windows"))]
  Scenario {
    name: "normalize / owned invalid input via owned receiver -> PathBuf",
    run: normalize_owned_invalid_consuming,
  },
  Scenario {
    name: "pipeline / dirty join via borrowed receiver -> PathBuf",
    run: join_normalize_owned,
  },
  Scenario {
    name: "pipeline / dirty join via owned receiver -> PathBuf",
    run: join_normalize_owned_consuming,
  },
  Scenario {
    name: "pipeline / dirty join via borrowed receiver -> String",
    run: join_normalize_slash_owned,
  },
  Scenario {
    name: "pipeline / dirty join via owned receiver -> String",
    run: join_normalize_slash_owned_consuming,
  },
  Scenario {
    name: "pipeline / clean join via borrowed receiver -> PathBuf",
    run: clean_join_normalize_owned,
  },
  Scenario {
    name: "pipeline / clean join via owned receiver -> PathBuf",
    run: clean_join_normalize_owned_consuming,
  },
  Scenario {
    name: "pipeline / clean join via borrowed receiver -> String",
    run: clean_join_normalize_slash_owned,
  },
  Scenario {
    name: "pipeline / clean join via owned receiver -> String",
    run: clean_join_normalize_slash_owned_consuming,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / absolute mixed separators -> normalized path",
    run: windows_normalize_absolute_mixed,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / canonical ordinary UNC -> normalized path",
    run: windows_normalize_canonical_unc,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / canonical verbatim UNC -> normalized path",
    run: windows_normalize_canonical_verbatim_unc,
  },
  Scenario {
    name: "absolutize / clean absolute input -> absolute path",
    run: absolutize_clean_absolute,
  },
  Scenario {
    name: "absolutize_with / absolute input + borrowed cwd -> absolute path",
    run: absolutize_with_absolute,
  },
  Scenario {
    name: "absolutize_with / relative input + borrowed cwd -> absolute path",
    run: absolutize_with_relative,
  },
  Scenario {
    name: "absolutize_with / relative input + owned cwd -> absolute path (setup excluded)",
    run: absolutize_with_relative_owned_base,
  },
  Scenario {
    name: "absolutize_with / clean relative input + owned cwd -> absolute path (setup excluded)",
    run: absolutize_with_clean_relative_owned_cwd,
  },
  Scenario {
    name: "relative / canonical native descendant -> natural result",
    run: relative_absolute,
  },
  Scenario {
    name: "relative / canonical native descendant -> PathBuf",
    run: relative_absolute_into_owned,
  },
  Scenario { name: "relative / relative inputs -> relative path", run: relative_relative },
  Scenario {
    name: "relative / dotted relative inputs -> relative path",
    run: relative_dotted_inputs,
  },
  Scenario {
    name: "relative / equal leading-parent inputs -> relative path",
    run: relative_equal_leading_parents,
  },
  Scenario {
    name: "relative / unequal leading-parent inputs -> relative path",
    run: relative_unequal_leading_parents,
  },
  Scenario {
    name: "relative / p99-depth equal leading-parent inputs -> relative path",
    run: relative_p99_depth_equal_leading_parents,
  },
  Scenario {
    name: "relative / p99-depth unequal leading-parent inputs -> relative path",
    run: relative_p99_depth_unequal_leading_parents,
  },
  Scenario {
    name: "relative / current-directory spellings -> relative path",
    run: relative_current_directory,
  },
  Scenario { name: "relative / dotted slow path -> relative path", run: relative_dotted_slow },
  #[cfg(any(target_family = "unix", target_family = "windows"))]
  Scenario {
    name: "relative / invalid-encoding absolute target -> relative path (setup excluded)",
    run: relative_invalid_encoding,
  },
  #[cfg(any(target_family = "unix", target_family = "windows"))]
  Scenario {
    name: "relative / invalid-encoding relative target -> relative path (setup excluded)",
    run: relative_invalid_relative_encoding,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / absolute forward-slash inputs -> relative path",
    run: windows_relative_absolute_forward,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / same drive-relative prefix -> relative path",
    run: windows_relative_same_drive_relative_prefix,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / different drive-relative prefix -> relative path",
    run: windows_relative_different_drive_relative_prefix,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / root-relative inputs -> relative path",
    run: windows_relative_root_relative,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / ordinary UNC same share -> relative path",
    run: windows_relative_unc_same_share,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / ordinary UNC different share -> relative path",
    run: windows_relative_unc_different_share,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / verbatim UNC same share -> relative path",
    run: windows_relative_verbatim_unc_same_share,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / verbatim UNC different share -> relative path",
    run: windows_relative_verbatim_unc_different_share,
  },
  Scenario { name: "to_slash / native path -> slash text", run: to_slash_native },
  #[cfg(any(target_family = "unix", target_family = "windows"))]
  Scenario {
    name: "to_slash_lossy / invalid encoding -> lossy slash text (setup excluded)",
    run: to_slash_lossy_invalid_encoding,
  },
  Scenario {
    name: "to_slash / owned valid input via borrowed receiver -> String",
    run: to_slash_owned_valid,
  },
  Scenario {
    name: "to_slash / owned valid input via owned receiver -> String",
    run: to_slash_owned_valid_consuming,
  },
  #[cfg(any(target_family = "unix", target_family = "windows"))]
  Scenario {
    name: "to_slash_lossy / owned invalid input via owned receiver -> String",
    run: to_slash_lossy_owned_invalid_consuming,
  },
  #[cfg(target_family = "windows")]
  Scenario {
    name: "Windows / already-forward path -> slash text",
    run: windows_to_slash_already_forward,
  },
  #[cfg(target_family = "windows")]
  Scenario { name: "Windows / mixed separators -> slash text", run: windows_to_slash_mixed },
  Scenario {
    name: "Rolldown / cwd descendant via natural relative result -> String",
    run: rolldown_relative_to_slash,
  },
  Scenario {
    name: "Rolldown / cwd descendant via requested PathBuf result -> String",
    run: rolldown_relative_to_consuming_slash,
  },
  Scenario {
    name: "Rolldown / cwd descendant via natural normalized result -> String",
    run: rolldown_relative_to_normalize_to_slash,
  },
  Scenario {
    name: "Rolldown / cwd descendant via requested PathBuf normalized result -> String",
    run: rolldown_relative_to_consuming_normalize_to_slash,
  },
  Scenario {
    name: "Rolldown / upward relation via natural relative result -> String",
    run: leading_parent_relative_to_slash,
  },
  Scenario {
    name: "Rolldown / upward relation via requested PathBuf result -> String",
    run: leading_parent_relative_to_consuming_slash,
  },
  Scenario {
    name: "Rolldown / upward relation via natural normalized result -> String",
    run: leading_parent_relative_to_normalize_to_slash,
  },
  Scenario {
    name: "Rolldown / upward relation via requested PathBuf normalized result -> String",
    run: leading_parent_relative_to_consuming_normalize_to_slash,
  },
  Scenario {
    name: "Rolldown / cwd descendant via natural relative result -> ArcStr",
    run: rolldown_relative_to_borrowed_slash_arcstr,
  },
  Scenario {
    name: "Rolldown / cwd descendant via natural relative result -> String -> ArcStr",
    run: rolldown_relative_to_owned_slash_arcstr,
  },
  Scenario {
    name: "Rolldown / cwd descendant via requested PathBuf result -> String -> ArcStr",
    run: rolldown_relative_to_consuming_slash_arcstr,
  },
  Scenario {
    name: "Rolldown / upward relation via natural relative result -> ArcStr",
    run: leading_parent_relative_to_borrowed_slash_arcstr,
  },
  Scenario {
    name: "Rolldown / upward relation via natural relative result -> String -> ArcStr",
    run: leading_parent_relative_to_owned_slash_arcstr,
  },
  Scenario {
    name: "Rolldown / upward relation via requested PathBuf result -> String -> ArcStr",
    run: leading_parent_relative_to_consuming_slash_arcstr,
  },
  Scenario {
    name: "Rolldown / sideEffects descendant via relative -> temporary text",
    run: rolldown_side_effects_descendant_relative,
  },
  Scenario {
    name: "Rolldown / sideEffects descendant via strip-prefix fallback -> temporary text",
    run: rolldown_side_effects_descendant_strip_prefix,
  },
  Scenario {
    name: "Rolldown / sideEffects upward via relative -> temporary text",
    run: rolldown_side_effects_upward_relative,
  },
  Scenario {
    name: "Rolldown / sideEffects upward via strip-prefix fallback -> temporary text",
    run: rolldown_side_effects_upward_strip_prefix,
  },
];

#[cfg(target_family = "windows")]
mod native_paths {
  pub const ABSOLUTE_BASE: &str = r"C:\workspace\rolldown\crates\rolldown";
  pub const ABSOLUTE_CLEAN: &str = r"C:\workspace\rolldown\crates\rolldown\src\bundle\bundle.rs";
  pub const ABSOLUTE_TARGET: &str =
    r"C:\workspace\rolldown\crates\rolldown\src\stages\generate_stage\mod.rs";
  pub const DIRTY: &str = r".\crates\\rolldown\src\.\module_loader\..\bundle\bundle.rs";
  pub const DOT_SEPARATOR: &str = r".\";
  pub const COLLAPSES_TO_CURRENT_DIRECTORY: &str = r"foo\..";
  pub const DOTTED_BASE: &str = r"C:\workspace\rolldown\crates\rolldown\.\src\stages\..\bundle";
  pub const DOTTED_TARGET: &str =
    r"C:\workspace\rolldown\crates\rolldown\src\utils\..\stages\generate_stage\mod.rs";
  pub const JOIN_RELATIVE: &str = r".\crates\rolldown\src\..\src\module_loader\module_task.rs";
  pub const JOIN_CLEAN: &str = r"crates\rolldown\src\module_loader\module_task.rs";
  pub const LEADING_PARENTS: &str = r"..\..\crates\rolldown\.\src\..\src\bundle\bundle.rs";
  pub const CANONICAL_LEADING_PARENTS: &str = r"..\..\chunks\shared.js";
  pub const RELATIVE_BASE: &str = r"crates\rolldown\src\module_loader";
  pub const RELATIVE_CLEAN_INPUT: &str = r"src\module_loader\module_task.rs";
  pub const RELATIVE_INPUT: &str = r".\src\stages\..\bundle\bundle.rs";
  pub const RELATIVE_TARGET: &str = r"crates\rolldown\src\stages\generate_stage\mod.rs";
  pub const RELATIVE_CURRENT_BASE: &str = "";
  pub const RELATIVE_CURRENT_TARGET: &str = ".";
  pub const RELATIVE_DOTTED_BASE: &str = r"dist\chunks\..\chunks";
  pub const RELATIVE_DOTTED_TARGET: &str = r"dist\.\assets\index-CQFG.js";
  pub const RELATIVE_EQUAL_PARENT_BASE: &str = r"..\..\dist\chunks";
  pub const RELATIVE_EQUAL_PARENT_TARGET: &str = r"..\..\dist\assets\index-CQFG.js";
  pub const RELATIVE_UNEQUAL_PARENT_BASE: &str = r"..\..\dist\chunks";
  pub const RELATIVE_UNEQUAL_PARENT_TARGET: &str = r"..\dist\assets\index-CQFG.js";
  pub const RELATIVE_P99_EQUAL_PARENT_BASE: &str = r"..\..\a\b\c\d\e\f\g\h\chunks";
  pub const RELATIVE_P99_EQUAL_PARENT_TARGET: &str = r"..\..\a\b\c\d\e\f\g\h\assets\index.js";
  pub const RELATIVE_P99_UNEQUAL_PARENT_BASE: &str = r"..\..\a\b\c\d\e\f\g\h\chunks";
  pub const RELATIVE_P99_UNEQUAL_PARENT_TARGET: &str = r"..\a\b\c\d\e\f\g\h\assets\index.js";
  pub const ROLLDOWN_CWD: &str = r"C:\workspace\rolldown";
  pub const ROLLDOWN_PARENT_BASE: &str = r"C:\workspace\rolldown\crates\rolldown\src\module_loader";
  pub const ROLLDOWN_PARENT_TARGET: &str =
    r"C:\workspace\rolldown\crates\rolldown\src\bundle\bundle.rs";
  pub const ROLLDOWN_TARGET: &str =
    r"C:\workspace\rolldown\crates\rolldown\src\module_loader\external_module_task.rs";
  pub const WINDOWS_FORWARD_BASE: &str = "C:/workspace/rolldown/crates/rolldown/src";
  pub const WINDOWS_FORWARD_TARGET: &str =
    "C:/workspace/rolldown/crates/rolldown/src/stages/generate_stage/mod.rs";
  pub const WINDOWS_DRIVE_RELATIVE_BASE: &str = r"C:dist\chunks";
  pub const WINDOWS_DRIVE_RELATIVE_TARGET: &str = r"C:dist\assets\index.js";
  pub const WINDOWS_DIFFERENT_DRIVE_RELATIVE_TARGET: &str = r"D:dist\assets\index.js";
  pub const WINDOWS_ROOT_RELATIVE_BASE: &str = r"\dist\chunks";
  pub const WINDOWS_ROOT_RELATIVE_TARGET: &str = r"\dist\assets\index.js";
  pub const WINDOWS_MIXED_ABSOLUTE: &str =
    r"C:/workspace/rolldown\crates/rolldown/src\bundle/.\bundle.rs";
  pub const WINDOWS_MIXED_SLASH: &str =
    r"C:/workspace\rolldown/crates\rolldown/src/bundle\bundle.rs";
  pub const UNC_CANONICAL: &str = r"\\server\share\packages\app\index.js";
  pub const UNC_SAME_SHARE_BASE: &str = r"\\server\share\dist\chunks";
  pub const UNC_SAME_SHARE_TARGET: &str = r"\\server\share\packages\app\index.js";
  pub const UNC_DIFFERENT_SHARE_TARGET: &str = r"\\server\other\packages\app\index.js";
  pub const VERBATIM_UNC_CANONICAL: &str = r"\\?\UNC\server\share\packages\app\index.js";
  pub const VERBATIM_UNC_SAME_SHARE_BASE: &str = r"\\?\UNC\server\share\dist\chunks";
  pub const VERBATIM_UNC_SAME_SHARE_TARGET: &str = r"\\?\UNC\server\share\packages\app\index.js";
  pub const VERBATIM_UNC_DIFFERENT_SHARE_TARGET: &str =
    r"\\?\UNC\server\other\packages\app\index.js";
}

#[cfg(not(target_family = "windows"))]
mod native_paths {
  pub const ABSOLUTE_BASE: &str = "/workspace/rolldown/crates/rolldown";
  pub const ABSOLUTE_CLEAN: &str = "/workspace/rolldown/crates/rolldown/src/bundle/bundle.rs";
  pub const ABSOLUTE_TARGET: &str =
    "/workspace/rolldown/crates/rolldown/src/stages/generate_stage/mod.rs";
  pub const DIRTY: &str = "./crates//rolldown/src/./module_loader/../bundle/bundle.rs";
  pub const DOT_SEPARATOR: &str = "./";
  pub const COLLAPSES_TO_CURRENT_DIRECTORY: &str = "foo/..";
  pub const DOTTED_BASE: &str = "/workspace/rolldown/crates/rolldown/./src/stages/../bundle";
  pub const DOTTED_TARGET: &str =
    "/workspace/rolldown/crates/rolldown/src/utils/../stages/generate_stage/mod.rs";
  pub const JOIN_RELATIVE: &str = "./crates/rolldown/src/../src/module_loader/module_task.rs";
  pub const JOIN_CLEAN: &str = "crates/rolldown/src/module_loader/module_task.rs";
  pub const LEADING_PARENTS: &str = "../../crates/rolldown/./src/../src/bundle/bundle.rs";
  pub const CANONICAL_LEADING_PARENTS: &str = "../../chunks/shared.js";
  pub const RELATIVE_BASE: &str = "crates/rolldown/src/module_loader";
  pub const RELATIVE_CLEAN_INPUT: &str = "src/module_loader/module_task.rs";
  pub const RELATIVE_INPUT: &str = "./src/stages/../bundle/bundle.rs";
  pub const RELATIVE_TARGET: &str = "crates/rolldown/src/stages/generate_stage/mod.rs";
  pub const RELATIVE_CURRENT_BASE: &str = "";
  pub const RELATIVE_CURRENT_TARGET: &str = ".";
  pub const RELATIVE_DOTTED_BASE: &str = "dist/chunks/../chunks";
  pub const RELATIVE_DOTTED_TARGET: &str = "dist/./assets/index-CQFG.js";
  pub const RELATIVE_EQUAL_PARENT_BASE: &str = "../../dist/chunks";
  pub const RELATIVE_EQUAL_PARENT_TARGET: &str = "../../dist/assets/index-CQFG.js";
  pub const RELATIVE_UNEQUAL_PARENT_BASE: &str = "../../dist/chunks";
  pub const RELATIVE_UNEQUAL_PARENT_TARGET: &str = "../dist/assets/index-CQFG.js";
  pub const RELATIVE_P99_EQUAL_PARENT_BASE: &str = "../../a/b/c/d/e/f/g/h/chunks";
  pub const RELATIVE_P99_EQUAL_PARENT_TARGET: &str = "../../a/b/c/d/e/f/g/h/assets/index.js";
  pub const RELATIVE_P99_UNEQUAL_PARENT_BASE: &str = "../../a/b/c/d/e/f/g/h/chunks";
  pub const RELATIVE_P99_UNEQUAL_PARENT_TARGET: &str = "../a/b/c/d/e/f/g/h/assets/index.js";
  pub const ROLLDOWN_CWD: &str = "/workspace/rolldown";
  pub const ROLLDOWN_PARENT_BASE: &str = "/workspace/rolldown/crates/rolldown/src/module_loader";
  pub const ROLLDOWN_PARENT_TARGET: &str =
    "/workspace/rolldown/crates/rolldown/src/bundle/bundle.rs";
  pub const ROLLDOWN_TARGET: &str =
    "/workspace/rolldown/crates/rolldown/src/module_loader/external_module_task.rs";
}

fn run_prepared<S>(
  mode: RunMode,
  setup: impl FnOnce() -> S,
  operation: impl FnOnce(S),
) -> AllocationStats {
  let prepared = setup();
  match mode {
    RunMode::Warm => {
      operation(black_box(prepared));
      AllocationStats::ZERO
    }
    RunMode::Measure => measure_once(move || operation(black_box(prepared))),
  }
}

fn normalize_clean(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || Path::new(native_paths::ABSOLUTE_CLEAN),
    |path| {
      let value = black_box(path).normalize();
      black_box(value);
    },
  )
}

fn normalize_dirty(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || Path::new(native_paths::DIRTY),
    |path| {
      let value = black_box(path).normalize();
      black_box(value);
    },
  )
}

fn normalize_current_directory_spelling(mode: RunMode, path: &'static str) -> AllocationStats {
  run_prepared(
    mode,
    || Path::new(path),
    |path| {
      let value = black_box(path).normalize();
      black_box(value);
    },
  )
}

fn normalize_empty(mode: RunMode) -> AllocationStats {
  normalize_current_directory_spelling(mode, "")
}

fn normalize_dot(mode: RunMode) -> AllocationStats {
  normalize_current_directory_spelling(mode, ".")
}

fn normalize_dot_separator(mode: RunMode) -> AllocationStats {
  normalize_current_directory_spelling(mode, native_paths::DOT_SEPARATOR)
}

fn normalize_collapsing_to_current_directory(mode: RunMode) -> AllocationStats {
  normalize_current_directory_spelling(mode, native_paths::COLLAPSES_TO_CURRENT_DIRECTORY)
}

fn normalize_leading_parents(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || Path::new(native_paths::LEADING_PARENTS),
    |path| {
      let value = black_box(path).normalize();
      black_box(value);
    },
  )
}

fn normalize_canonical_leading_parents(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || Path::new(native_paths::CANONICAL_LEADING_PARENTS),
    |path| {
      let value = black_box(path).normalize();
      black_box(value);
    },
  )
}

#[cfg(target_family = "unix")]
fn invalid_clean_path() -> PathBuf {
  use std::os::unix::ffi::OsStringExt as _;

  PathBuf::from(OsString::from_vec(
    b"/workspace/rolldown/crates/rolldown/src/module_loader/invalid-\xff.rs".to_vec(),
  ))
}

#[cfg(target_family = "windows")]
fn invalid_clean_path() -> PathBuf {
  use std::os::windows::ffi::OsStringExt as _;

  let mut units: Vec<u16> =
    r"C:\workspace\rolldown\crates\rolldown\src\module_loader\invalid-".encode_utf16().collect();
  units.push(0xd800);
  units.extend(".rs".encode_utf16());
  PathBuf::from(OsString::from_wide(&units))
}

#[cfg(target_family = "unix")]
fn invalid_relative_path() -> PathBuf {
  use std::os::unix::ffi::OsStringExt as _;

  PathBuf::from(OsString::from_vec(b"dist/assets/invalid-\xff.js".to_vec()))
}

#[cfg(target_family = "windows")]
fn invalid_relative_path() -> PathBuf {
  use std::os::windows::ffi::OsStringExt as _;

  let mut units: Vec<u16> = r"dist\assets\invalid-".encode_utf16().collect();
  units.push(0xd800);
  units.extend(".js".encode_utf16());
  PathBuf::from(OsString::from_wide(&units))
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
fn normalize_invalid_clean(mode: RunMode) -> AllocationStats {
  run_prepared(mode, invalid_clean_path, |path| {
    let value = black_box(path.as_path()).normalize();
    black_box(value);
  })
}

fn normalize_owned_clean(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ABSOLUTE_CLEAN),
    |path| {
      let value = black_box(path.as_path()).normalize().into_owned();
      black_box(value);
    },
  )
}

fn normalize_owned_case(mode: RunMode, input: &'static str) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(input),
    |path| {
      let value = black_box(path.as_path()).normalize().into_owned();
      black_box(value);
    },
  )
}

fn normalize_owned_case_consuming(mode: RunMode, input: &'static str) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(input),
    |path| {
      let value = black_box(path).into_normalized();
      black_box(value);
    },
  )
}

fn normalize_owned_clean_consuming(mode: RunMode) -> AllocationStats {
  normalize_owned_case_consuming(mode, native_paths::ABSOLUTE_CLEAN)
}

fn normalize_owned_dirty(mode: RunMode) -> AllocationStats {
  normalize_owned_case(mode, native_paths::DIRTY)
}

fn normalize_owned_dirty_consuming(mode: RunMode) -> AllocationStats {
  normalize_owned_case_consuming(mode, native_paths::DIRTY)
}

fn normalize_owned_dot(mode: RunMode) -> AllocationStats {
  normalize_owned_case(mode, ".")
}

fn normalize_owned_dot_consuming(mode: RunMode) -> AllocationStats {
  normalize_owned_case_consuming(mode, ".")
}

fn normalize_owned_collapse(mode: RunMode) -> AllocationStats {
  normalize_owned_case(mode, native_paths::COLLAPSES_TO_CURRENT_DIRECTORY)
}

fn normalize_owned_collapse_consuming(mode: RunMode) -> AllocationStats {
  normalize_owned_case_consuming(mode, native_paths::COLLAPSES_TO_CURRENT_DIRECTORY)
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
fn normalize_owned_invalid(mode: RunMode) -> AllocationStats {
  run_prepared(mode, invalid_clean_path, |path| {
    black_box(black_box(path.as_path()).normalize().into_owned());
  })
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
fn normalize_owned_invalid_consuming(mode: RunMode) -> AllocationStats {
  run_prepared(mode, invalid_clean_path, |path| {
    black_box(black_box(path).into_normalized());
  })
}

fn join_normalize_owned(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ROLLDOWN_CWD),
    |base| {
      let base = black_box(base);
      let joined = base.join(black_box(Path::new(native_paths::JOIN_RELATIVE)));
      let normalized = joined.normalize().into_owned();
      black_box(normalized);
    },
  )
}

fn join_normalize_owned_consuming(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ROLLDOWN_CWD),
    |base| {
      let joined = black_box(base).join(black_box(Path::new(native_paths::JOIN_RELATIVE)));
      black_box(joined.into_normalized());
    },
  )
}

fn join_normalize_slash_owned(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ROLLDOWN_CWD),
    |base| {
      let base = black_box(base);
      let joined = base.join(black_box(Path::new(native_paths::JOIN_RELATIVE)));
      let normalized = joined.normalize();
      let slash = normalized.to_slash().into_owned();
      black_box(slash);
    },
  )
}

fn join_normalize_slash_owned_consuming(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ROLLDOWN_CWD),
    |base| {
      let joined = black_box(base).join(black_box(Path::new(native_paths::JOIN_RELATIVE)));
      black_box(joined.into_normalized().into_slash());
    },
  )
}

fn clean_join_normalize_owned(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ROLLDOWN_CWD),
    |base| {
      let joined = black_box(base).join(black_box(Path::new(native_paths::JOIN_CLEAN)));
      black_box(joined.normalize().into_owned());
    },
  )
}

fn clean_join_normalize_owned_consuming(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ROLLDOWN_CWD),
    |base| {
      let joined = black_box(base).join(black_box(Path::new(native_paths::JOIN_CLEAN)));
      black_box(joined.into_normalized());
    },
  )
}

fn clean_join_normalize_slash_owned(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ROLLDOWN_CWD),
    |base| {
      let joined = black_box(base).join(black_box(Path::new(native_paths::JOIN_CLEAN)));
      black_box(joined.normalize().to_slash().into_owned());
    },
  )
}

fn clean_join_normalize_slash_owned_consuming(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ROLLDOWN_CWD),
    |base| {
      let joined = black_box(base).join(black_box(Path::new(native_paths::JOIN_CLEAN)));
      black_box(joined.into_normalized().into_slash());
    },
  )
}

#[cfg(target_family = "windows")]
fn windows_normalize_absolute_mixed(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || Path::new(native_paths::WINDOWS_MIXED_ABSOLUTE),
    |path| {
      let value = black_box(path).normalize();
      black_box(value);
    },
  )
}

#[cfg(target_family = "windows")]
fn windows_normalize_case(mode: RunMode, path: &'static str) -> AllocationStats {
  run_prepared(
    mode,
    || Path::new(path),
    |path| {
      let value = black_box(path).normalize();
      black_box(value);
    },
  )
}

#[cfg(target_family = "windows")]
fn windows_normalize_canonical_unc(mode: RunMode) -> AllocationStats {
  windows_normalize_case(mode, native_paths::UNC_CANONICAL)
}

#[cfg(target_family = "windows")]
fn windows_normalize_canonical_verbatim_unc(mode: RunMode) -> AllocationStats {
  windows_normalize_case(mode, native_paths::VERBATIM_UNC_CANONICAL)
}

fn absolutize_clean_absolute(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || Path::new(native_paths::ABSOLUTE_CLEAN),
    |path| {
      let value = black_box(path).absolutize();
      black_box(value);
    },
  )
}

fn absolutize_with_absolute(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let cwd = black_box(Path::new(native_paths::ABSOLUTE_BASE));
      let value = black_box(Path::new(native_paths::ABSOLUTE_CLEAN)).absolutize_with(cwd);
      black_box(value);
    },
  )
}

fn absolutize_with_relative(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let cwd = black_box(Path::new(native_paths::ABSOLUTE_BASE));
      let value = black_box(Path::new(native_paths::RELATIVE_INPUT)).absolutize_with(cwd);
      black_box(value);
    },
  )
}

fn absolutize_with_relative_owned_base(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ABSOLUTE_BASE),
    |cwd| {
      let value =
        black_box(Path::new(native_paths::RELATIVE_INPUT)).absolutize_with(black_box(cwd));
      black_box(value);
    },
  )
}

fn absolutize_with_clean_relative_owned_cwd(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ABSOLUTE_BASE),
    |cwd| {
      let value =
        black_box(Path::new(native_paths::RELATIVE_CLEAN_INPUT)).absolutize_with(black_box(cwd));
      black_box(value);
    },
  )
}

fn relative_absolute(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let value = black_box(Path::new(native_paths::ABSOLUTE_TARGET))
        .relative(black_box(Path::new(native_paths::ABSOLUTE_BASE)));
      black_box(value);
    },
  )
}

fn relative_absolute_into_owned(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let value = black_box(Path::new(native_paths::ABSOLUTE_TARGET))
        .relative(black_box(Path::new(native_paths::ABSOLUTE_BASE)))
        .into_owned();
      black_box(value);
    },
  )
}

fn relative_relative(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let value = black_box(Path::new(native_paths::RELATIVE_TARGET))
        .relative(black_box(Path::new(native_paths::RELATIVE_BASE)));
      black_box(value);
    },
  )
}

fn relative_case(mode: RunMode, target: &'static str, base: &'static str) -> AllocationStats {
  run_prepared(
    mode,
    || (Path::new(target), Path::new(base)),
    |(target, base)| {
      let value = black_box(target).relative(black_box(base));
      black_box(value);
    },
  )
}

fn relative_dotted_inputs(mode: RunMode) -> AllocationStats {
  relative_case(mode, native_paths::RELATIVE_DOTTED_TARGET, native_paths::RELATIVE_DOTTED_BASE)
}

fn relative_equal_leading_parents(mode: RunMode) -> AllocationStats {
  relative_case(
    mode,
    native_paths::RELATIVE_EQUAL_PARENT_TARGET,
    native_paths::RELATIVE_EQUAL_PARENT_BASE,
  )
}

fn relative_unequal_leading_parents(mode: RunMode) -> AllocationStats {
  relative_case(
    mode,
    native_paths::RELATIVE_UNEQUAL_PARENT_TARGET,
    native_paths::RELATIVE_UNEQUAL_PARENT_BASE,
  )
}

fn relative_p99_depth_equal_leading_parents(mode: RunMode) -> AllocationStats {
  relative_case(
    mode,
    native_paths::RELATIVE_P99_EQUAL_PARENT_TARGET,
    native_paths::RELATIVE_P99_EQUAL_PARENT_BASE,
  )
}

fn relative_p99_depth_unequal_leading_parents(mode: RunMode) -> AllocationStats {
  relative_case(
    mode,
    native_paths::RELATIVE_P99_UNEQUAL_PARENT_TARGET,
    native_paths::RELATIVE_P99_UNEQUAL_PARENT_BASE,
  )
}

fn relative_current_directory(mode: RunMode) -> AllocationStats {
  relative_case(mode, native_paths::RELATIVE_CURRENT_TARGET, native_paths::RELATIVE_CURRENT_BASE)
}

fn relative_dotted_slow(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let value = black_box(Path::new(native_paths::DOTTED_TARGET))
        .relative(black_box(Path::new(native_paths::DOTTED_BASE)));
      black_box(value);
    },
  )
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
fn relative_invalid_encoding(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (invalid_clean_path(), PathBuf::from(native_paths::ROLLDOWN_CWD)),
    |(target, base)| {
      let value = black_box(target.as_path()).relative(black_box(base.as_path()));
      black_box(value);
    },
  )
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
fn relative_invalid_relative_encoding(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (invalid_relative_path(), PathBuf::from(native_paths::RELATIVE_DOTTED_BASE)),
    |(target, base)| {
      let value = black_box(target.as_path()).relative(black_box(base.as_path()));
      black_box(value);
    },
  )
}

#[cfg(target_family = "windows")]
fn windows_relative_absolute_forward(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let value = black_box(Path::new(native_paths::WINDOWS_FORWARD_TARGET))
        .relative(black_box(Path::new(native_paths::WINDOWS_FORWARD_BASE)));
      black_box(value);
    },
  )
}

#[cfg(target_family = "windows")]
fn windows_relative_case(
  mode: RunMode,
  target: &'static str,
  base: &'static str,
) -> AllocationStats {
  run_prepared(
    mode,
    || (Path::new(target), Path::new(base)),
    |(target, base)| {
      let value = black_box(target).relative(black_box(base));
      black_box(value);
    },
  )
}

#[cfg(target_family = "windows")]
fn windows_relative_same_drive_relative_prefix(mode: RunMode) -> AllocationStats {
  windows_relative_case(
    mode,
    native_paths::WINDOWS_DRIVE_RELATIVE_TARGET,
    native_paths::WINDOWS_DRIVE_RELATIVE_BASE,
  )
}

#[cfg(target_family = "windows")]
fn windows_relative_different_drive_relative_prefix(mode: RunMode) -> AllocationStats {
  windows_relative_case(
    mode,
    native_paths::WINDOWS_DIFFERENT_DRIVE_RELATIVE_TARGET,
    native_paths::WINDOWS_DRIVE_RELATIVE_BASE,
  )
}

#[cfg(target_family = "windows")]
fn windows_relative_root_relative(mode: RunMode) -> AllocationStats {
  windows_relative_case(
    mode,
    native_paths::WINDOWS_ROOT_RELATIVE_TARGET,
    native_paths::WINDOWS_ROOT_RELATIVE_BASE,
  )
}

#[cfg(target_family = "windows")]
fn windows_relative_unc_same_share(mode: RunMode) -> AllocationStats {
  windows_relative_case(
    mode,
    native_paths::UNC_SAME_SHARE_TARGET,
    native_paths::UNC_SAME_SHARE_BASE,
  )
}

#[cfg(target_family = "windows")]
fn windows_relative_unc_different_share(mode: RunMode) -> AllocationStats {
  windows_relative_case(
    mode,
    native_paths::UNC_DIFFERENT_SHARE_TARGET,
    native_paths::UNC_SAME_SHARE_BASE,
  )
}

#[cfg(target_family = "windows")]
fn windows_relative_verbatim_unc_same_share(mode: RunMode) -> AllocationStats {
  windows_relative_case(
    mode,
    native_paths::VERBATIM_UNC_SAME_SHARE_TARGET,
    native_paths::VERBATIM_UNC_SAME_SHARE_BASE,
  )
}

#[cfg(target_family = "windows")]
fn windows_relative_verbatim_unc_different_share(mode: RunMode) -> AllocationStats {
  windows_relative_case(
    mode,
    native_paths::VERBATIM_UNC_DIFFERENT_SHARE_TARGET,
    native_paths::VERBATIM_UNC_SAME_SHARE_BASE,
  )
}

fn to_slash_native(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || Path::new(native_paths::ABSOLUTE_CLEAN),
    |path| {
      let value = black_box(path).to_slash();
      black_box(value);
    },
  )
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
fn to_slash_lossy_invalid_encoding(mode: RunMode) -> AllocationStats {
  run_prepared(mode, invalid_clean_path, |path| {
    let value = black_box(path.as_path()).to_slash_lossy();
    black_box(value);
  })
}

fn to_slash_owned_valid(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ABSOLUTE_CLEAN),
    |path| {
      black_box(black_box(path.as_path()).to_slash().into_owned());
    },
  )
}

fn to_slash_owned_valid_consuming(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || PathBuf::from(native_paths::ABSOLUTE_CLEAN),
    |path| {
      black_box(black_box(path).into_slash());
    },
  )
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
fn to_slash_lossy_owned_invalid_consuming(mode: RunMode) -> AllocationStats {
  run_prepared(mode, invalid_clean_path, |path| {
    black_box(black_box(path).into_slash_lossy());
  })
}

#[cfg(target_family = "windows")]
fn windows_to_slash_already_forward(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || Path::new(native_paths::WINDOWS_FORWARD_TARGET),
    |path| {
      let value = black_box(path).to_slash();
      black_box(value);
    },
  )
}

#[cfg(target_family = "windows")]
fn windows_to_slash_mixed(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || Path::new(native_paths::WINDOWS_MIXED_SLASH),
    |path| {
      let value = black_box(path).to_slash();
      black_box(value);
    },
  )
}

fn rolldown_relative_to_slash(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_CWD)));
      let slash = relative.to_slash().into_owned();
      black_box(slash);
    },
  )
}

fn rolldown_relative_to_consuming_slash(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_CWD)));
      black_box(relative.into_owned().into_slash());
    },
  )
}

fn rolldown_relative_to_borrowed_slash_arcstr(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_CWD)));
      black_box(ArcStr::from(relative.to_slash()));
    },
  )
}

fn rolldown_relative_to_owned_slash_arcstr(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_CWD)));
      black_box(ArcStr::from(relative.to_slash().into_owned()));
    },
  )
}

fn rolldown_relative_to_consuming_slash_arcstr(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_CWD)));
      black_box(ArcStr::from(relative.into_owned().into_slash()));
    },
  )
}

fn rolldown_side_effects_descendant_relative(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let target = black_box(Path::new(native_paths::ROLLDOWN_TARGET));
      let base = black_box(Path::new(native_paths::ROLLDOWN_CWD));
      let relative = target.relative(base);
      black_box(relative.to_str().expect("benchmark paths are valid UTF-8"));
    },
  )
}

fn rolldown_side_effects_descendant_strip_prefix(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let target = black_box(Path::new(native_paths::ROLLDOWN_TARGET));
      let base = black_box(Path::new(native_paths::ROLLDOWN_CWD));
      let relative = match target.strip_prefix(base) {
        Ok(relative) => Cow::Borrowed(relative),
        Err(_) => target.relative(base),
      };
      black_box(relative.to_str().expect("benchmark paths are valid UTF-8"));
    },
  )
}

fn rolldown_side_effects_upward_relative(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let target = black_box(Path::new(native_paths::ROLLDOWN_PARENT_TARGET));
      let base = black_box(Path::new(native_paths::ROLLDOWN_PARENT_BASE));
      let relative = target.relative(base);
      black_box(relative.to_str().expect("benchmark paths are valid UTF-8"));
    },
  )
}

fn rolldown_side_effects_upward_strip_prefix(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let target = black_box(Path::new(native_paths::ROLLDOWN_PARENT_TARGET));
      let base = black_box(Path::new(native_paths::ROLLDOWN_PARENT_BASE));
      let relative = match target.strip_prefix(base) {
        Ok(relative) => Cow::Borrowed(relative),
        Err(_) => target.relative(base),
      };
      black_box(relative.to_str().expect("benchmark paths are valid UTF-8"));
    },
  )
}

fn rolldown_relative_to_normalize_to_slash(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_CWD)));
      let normalized = relative.normalize();
      let slash = normalized.to_slash().into_owned();
      black_box(slash);
    },
  )
}

fn rolldown_relative_to_consuming_normalize_to_slash(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_CWD)));
      black_box(relative.into_owned().into_normalized().into_slash());
    },
  )
}

fn leading_parent_relative_to_slash(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_PARENT_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_PARENT_BASE)));
      black_box(relative.to_slash().into_owned());
    },
  )
}

fn leading_parent_relative_to_normalize_to_slash(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_PARENT_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_PARENT_BASE)));
      let normalized = relative.normalize();
      let slash = normalized.to_slash().into_owned();
      black_box(slash);
    },
  )
}

fn leading_parent_relative_to_consuming_normalize_to_slash(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_PARENT_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_PARENT_BASE)));
      black_box(relative.into_owned().into_normalized().into_slash());
    },
  )
}

fn leading_parent_relative_to_borrowed_slash_arcstr(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_PARENT_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_PARENT_BASE)));
      black_box(ArcStr::from(relative.to_slash()));
    },
  )
}

fn leading_parent_relative_to_owned_slash_arcstr(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_PARENT_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_PARENT_BASE)));
      black_box(ArcStr::from(relative.to_slash().into_owned()));
    },
  )
}

fn leading_parent_relative_to_consuming_slash(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_PARENT_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_PARENT_BASE)));
      black_box(relative.into_owned().into_slash());
    },
  )
}

fn leading_parent_relative_to_consuming_slash_arcstr(mode: RunMode) -> AllocationStats {
  run_prepared(
    mode,
    || (),
    |()| {
      let relative = black_box(Path::new(native_paths::ROLLDOWN_PARENT_TARGET))
        .relative(black_box(Path::new(native_paths::ROLLDOWN_PARENT_BASE)));
      black_box(ArcStr::from(relative.into_owned().into_slash()));
    },
  )
}

fn reset_counters() {
  ALLOC_CALLS.store(0, Ordering::Relaxed);
  ALLOC_ZEROED_CALLS.store(0, Ordering::Relaxed);
  REALLOC_CALLS.store(0, Ordering::Relaxed);
  ALLOC_BYTES.store(0, Ordering::Relaxed);
  REALLOC_NEW_BYTES.store(0, Ordering::Relaxed);
}

fn measure_once(run: impl FnOnce()) -> AllocationStats {
  reset_counters();
  let tracking = TrackingGuard::begin();
  run();
  drop(tracking);
  AllocationStats::load()
}

fn measure_scenario(scenario: Scenario) -> Measurement {
  // Warm lazy runtime and allocator state with freshly prepared input and no tracked region.
  (scenario.run)(RunMode::Warm);

  let expected = (scenario.run)(RunMode::Measure);
  for sample in 2..=MEASUREMENTS {
    let actual = (scenario.run)(RunMode::Measure);
    assert_eq!(
      actual, expected,
      "scenario {:?} produced non-deterministic allocation stats at sample {sample}",
      scenario.name
    );
  }

  Measurement { name: scenario.name, stats: expected }
}

fn measure_all() -> Vec<Measurement> {
  SCENARIOS.iter().copied().map(measure_scenario).collect()
}

fn render_snapshot(measurements: &[Measurement], cwd_shape: CwdShape) -> String {
  let mut output = String::new();
  writeln!(output, "# sugar_path allocation snapshot\n").unwrap();
  writeln!(
    output,
    "Platform: `{}/{}`; target environment: `{}`; profile: `{}`; configuration: `{}`; native separator: `{}`; measurement cwd: {} encoded bytes / {} components.\n",
    std::env::consts::OS,
    std::env::consts::ARCH,
    TARGET_ENVIRONMENT,
    if cfg!(debug_assertions) { "debug" } else { "release" },
    "cached_current_dir",
    std::path::MAIN_SEPARATOR,
    cwd_shape.encoded_bytes,
    cwd_shape.components
  )
  .unwrap();
  writeln!(output, "Each row measures one operation after allocation-capable setup and an untracked warm-up. Every row was reproduced identically {MEASUREMENTS} times.\n").unwrap();
  writeln!(output, "## Allocation calls (hard gate)\n").unwrap();
  writeln!(output, "| Scenario | `alloc` | `alloc_zeroed` | `realloc` |").unwrap();
  writeln!(output, "| --- | ---: | ---: | ---: |").unwrap();
  for measurement in measurements {
    writeln!(
      output,
      "| {} | {} | {} | {} |",
      measurement.name,
      measurement.stats.alloc_calls,
      measurement.stats.alloc_zeroed_calls,
      measurement.stats.realloc_calls
    )
    .unwrap();
  }
  writeln!(output, "\n## Requested bytes (platform-specific evidence)\n").unwrap();
  writeln!(output, "| Scenario | alloc + alloc_zeroed bytes | realloc new bytes |").unwrap();
  writeln!(output, "| --- | ---: | ---: |").unwrap();
  for measurement in measurements {
    writeln!(
      output,
      "| {} | {} | {} |",
      measurement.name, measurement.stats.alloc_bytes, measurement.stats.realloc_new_bytes
    )
    .unwrap();
  }
  output
}

enum Mode {
  Print,
  Check(PathBuf),
  Write(PathBuf),
}

fn usage() -> &'static str {
  "usage: track_allocations [--check PATH | --write PATH]"
}

fn parse_mode() -> Result<Mode, String> {
  let mut args = std::env::args_os().skip(1);
  let Some(flag) = args.next() else {
    return Ok(Mode::Print);
  };

  if flag == "--help" || flag == "-h" {
    return Err(usage().to_owned());
  }

  let path = args.next().ok_or_else(|| format!("missing path\n{}", usage()))?;
  if args.next().is_some() {
    return Err(format!("unexpected extra argument\n{}", usage()));
  }

  if flag == "--check" {
    Ok(Mode::Check(PathBuf::from(path)))
  } else if flag == "--write" {
    Ok(Mode::Write(PathBuf::from(path)))
  } else {
    Err(format!("unknown option {:?}\n{}", flag, usage()))
  }
}

fn platform_line(snapshot: &str) -> Option<&str> {
  snapshot.lines().find(|line| line.starts_with("Platform: `"))
}

fn hard_gate_section(snapshot: &str) -> Option<String> {
  let mut section = String::new();
  let mut in_section = false;
  for line in snapshot.lines() {
    if line == "## Requested bytes (platform-specific evidence)" {
      return in_section.then_some(section);
    }
    if !in_section {
      if line != "## Allocation calls (hard gate)" {
        continue;
      }
      in_section = true;
    }
    if !section.is_empty() {
      section.push('\n');
    }
    section.push_str(line);
  }
  None
}

fn check_snapshot(path: &Path, actual: &str) -> Result<(), String> {
  let expected = fs::read_to_string(path)
    .map_err(|error| format!("failed to read snapshot {}: {error}", path.display()))?;

  let expected_platform = platform_line(&expected)
    .ok_or_else(|| format!("snapshot {} has no platform line", path.display()))?;
  let actual_platform =
    platform_line(actual).expect("generated snapshots always have a platform line");
  if expected_platform != actual_platform {
    return Err(format!(
      "snapshot platform mismatch in {}\nexpected: {expected_platform}\nactual:   {actual_platform}",
      path.display()
    ));
  }

  let expected_gate = hard_gate_section(&expected)
    .ok_or_else(|| format!("snapshot {} has no hard-gate section", path.display()))?;
  let actual_gate =
    hard_gate_section(actual).expect("generated snapshots always have a hard-gate section");
  if expected_gate != actual_gate {
    return Err(format!(
      "allocation call counts differ from {}\n\nGenerated snapshot:\n{actual}",
      path.display()
    ));
  }

  if !expected.lines().eq(actual.lines()) {
    eprintln!("allocation call counts match {}; requested-byte evidence changed", path.display());
  }
  Ok(())
}

fn run() -> Result<(), String> {
  let mode = parse_mode()?;
  let measurement_cwd = MeasurementCwd::enter()?;
  let cwd_shape = measurement_cwd.shape;
  let measurements = measure_all();
  measurement_cwd.restore()?;
  let snapshot = render_snapshot(&measurements, cwd_shape);
  match mode {
    Mode::Print => print!("{snapshot}"),
    Mode::Check(path) => check_snapshot(&path, &snapshot)?,
    Mode::Write(path) => fs::write(&path, snapshot)
      .map_err(|error| format!("failed to write snapshot {}: {error}", path.display()))?,
  }
  Ok(())
}

fn main() -> ExitCode {
  match run() {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("{error}");
      ExitCode::FAILURE
    }
  }
}

#[cfg(test)]
mod tests {
  use super::hard_gate_section;

  #[test]
  fn hard_gate_parser_accepts_lf_and_crlf_snapshots() {
    let lf = "header\n## Allocation calls (hard gate)\n\n| row | 1 |\n\n## Requested bytes (platform-specific evidence)\nbytes\n";
    let crlf = lf.replace('\n', "\r\n");
    assert_eq!(hard_gate_section(lf), hard_gate_section(&crlf));
    assert!(hard_gate_section(lf).is_some());
  }
}
