#![allow(dead_code)]

#[cfg(any(unix, windows))]
use std::ffi::OsString;
#[cfg(any(unix, windows))]
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub struct PathCase {
  pub name: &'static str,
  pub path: &'static str,
}

#[derive(Clone, Copy)]
pub struct RelativeCase {
  pub name: &'static str,
  pub target: &'static str,
  pub base: &'static str,
}

#[cfg(unix)]
pub fn invalid_unicode_path() -> PathBuf {
  use std::os::unix::ffi::OsStringExt;

  PathBuf::from(OsString::from_vec(
    b"/workspace/rolldown/crates/rolldown/src/module_loader/invalid-\xff.rs".to_vec(),
  ))
}

#[cfg(windows)]
pub fn invalid_unicode_path() -> PathBuf {
  use std::os::windows::ffi::OsStringExt;

  let mut units: Vec<u16> =
    r"C:\workspace\rolldown\crates\rolldown\src\module_loader\invalid-".encode_utf16().collect();
  units.push(0xd800);
  units.extend(".rs".encode_utf16());
  PathBuf::from(OsString::from_wide(&units))
}

#[cfg(not(target_family = "windows"))]
pub const ROLLDOWN_ROOT: &str = "/workspace/rolldown";
#[cfg(target_family = "windows")]
pub const ROLLDOWN_ROOT: &str = r"C:\workspace\rolldown";

/// Public paths sampled from Rolldown's git tree at commit b9823050b.
///
/// Its 12,287 tracked repository-relative paths have 75 bytes and 7 components
/// at p50, 102 bytes and 9 components at p90, and 129 bytes and 10 components
/// at p99. The Unix benchmark values add the 20-byte /workspace/rolldown/ prefix.
#[cfg(not(target_family = "windows"))]
pub const ROLLDOWN_PATHS: &[PathCase] = &[
  PathCase {
    name: "short_source",
    path: "/workspace/rolldown/crates/rolldown/src/bundle/bundle.rs",
  },
  PathCase {
    name: "fixture_p50",
    path: "/workspace/rolldown/crates/rolldown/tests/esbuild/dce/disable_tree_shaking/keep-me/package.json",
  },
  PathCase {
    name: "module_loader_hot",
    path: "/workspace/rolldown/crates/rolldown/src/module_loader/external_module_task.rs",
  },
  PathCase {
    name: "fixture_p90",
    path: "/workspace/rolldown/crates/rolldown/tests/rolldown/function/chunk_optimization/dynamic_already_loaded_multi_entry/main2.js",
  },
  PathCase {
    name: "fixture_p95",
    path: "/workspace/rolldown/crates/rolldown/tests/rolldown/topics/live_bindings/default_export_binding_in_common_chunks_cjs/artifacts.snap",
  },
  PathCase {
    name: "fixture_p99",
    path: "/workspace/rolldown/crates/rolldown/tests/esbuild/dce/package_json_side_effects_array_keep_module_implicit_module/node_modules/demo-pkg/index-main.js",
  },
];

#[cfg(target_family = "windows")]
pub const ROLLDOWN_PATHS: &[PathCase] = &[
  PathCase {
    name: "short_source",
    path: r"C:\workspace\rolldown\crates\rolldown\src\bundle\bundle.rs",
  },
  PathCase {
    name: "fixture_p50",
    path: r"C:\workspace\rolldown\crates\rolldown\tests\esbuild\dce\disable_tree_shaking\keep-me\package.json",
  },
  PathCase {
    name: "module_loader_hot",
    path: r"C:\workspace\rolldown\crates\rolldown\src\module_loader\external_module_task.rs",
  },
  PathCase {
    name: "fixture_p90",
    path: r"C:\workspace\rolldown\crates\rolldown\tests\rolldown\function\chunk_optimization\dynamic_already_loaded_multi_entry\main2.js",
  },
  PathCase {
    name: "fixture_p95",
    path: r"C:\workspace\rolldown\crates\rolldown\tests\rolldown\topics\live_bindings\default_export_binding_in_common_chunks_cjs\artifacts.snap",
  },
  PathCase {
    name: "fixture_p99",
    path: r"C:\workspace\rolldown\crates\rolldown\tests\esbuild\dce\package_json_side_effects_array_keep_module_implicit_module\node_modules\demo-pkg\index-main.js",
  },
];

#[cfg(target_family = "windows")]
pub const WINDOWS_SLASH_CASES: &[PathCase] = &[
  PathCase {
    name: "already_forward",
    path: "C:/workspace/rolldown/crates/rolldown/src/bundle/bundle.rs",
  },
  PathCase {
    name: "mixed_separators",
    path: r"C:\workspace\rolldown\crates/rolldown\src/bundle\bundle.rs",
  },
];

#[cfg(not(target_family = "windows"))]
pub const DIRTY_PATHS: &[PathCase] = &[
  PathCase {
    name: "dot_component",
    path: "/workspace/rolldown/crates/rolldown/src/./module_loader/module_task.rs",
  },
  PathCase {
    name: "parent_cancellation",
    path: "/workspace/rolldown/crates/rolldown/src/module_loader/../bundle/bundle.rs",
  },
  PathCase {
    name: "duplicate_separator",
    path: "/workspace/rolldown/crates//rolldown/src/bundle/bundle.rs",
  },
  PathCase { name: "trailing_separator", path: "/workspace/rolldown/crates/rolldown/src/bundle/" },
];

#[cfg(target_family = "windows")]
pub const DIRTY_PATHS: &[PathCase] = &[
  PathCase {
    name: "forward_slashes",
    path: "C:/workspace/rolldown/crates/rolldown/src/bundle/bundle.rs",
  },
  PathCase {
    name: "mixed_separators",
    path: r"C:\workspace\rolldown\crates/rolldown\src/bundle\bundle.rs",
  },
  PathCase {
    name: "dot_component",
    path: r"C:\workspace\rolldown\crates\rolldown\src\.\module_loader\module_task.rs",
  },
  PathCase {
    name: "parent_cancellation",
    path: r"C:\workspace\rolldown\crates\rolldown\src\module_loader\..\bundle\bundle.rs",
  },
  PathCase {
    name: "duplicate_separator",
    path: r"C:\workspace\rolldown\crates\\rolldown\src\bundle\bundle.rs",
  },
  PathCase {
    name: "trailing_separator",
    path: r"C:\workspace\rolldown\crates\rolldown\src\bundle\",
  },
];

#[cfg(not(target_family = "windows"))]
pub const CANONICAL_LEADING_PARENTS: PathCase =
  PathCase { name: "leading_parents", path: "../../chunks/shared.js" };
#[cfg(target_family = "windows")]
pub const CANONICAL_LEADING_PARENTS: PathCase =
  PathCase { name: "leading_parents", path: r"..\..\chunks\shared.js" };

#[cfg(not(target_family = "windows"))]
pub const CURRENT_DIRECTORY_CASES: &[PathCase] = &[
  PathCase { name: "empty", path: "" },
  PathCase { name: "dot", path: "." },
  PathCase { name: "dot_separator", path: "./" },
  PathCase { name: "collapsing", path: "foo/.." },
];

#[cfg(target_family = "windows")]
pub const CURRENT_DIRECTORY_CASES: &[PathCase] = &[
  PathCase { name: "empty", path: "" },
  PathCase { name: "dot", path: "." },
  PathCase { name: "dot_separator", path: r".\" },
  PathCase { name: "collapsing", path: r"foo\.." },
];

#[cfg(not(target_family = "windows"))]
pub const LEADING_PARENT_SCAN_CASES: &[PathCase] = &[
  PathCase {
    name: "clean_68b",
    path: "../../crates/rolldown_plugin_vite_resolve/src/vite_resolve_plugin.rs",
  },
  PathCase {
    name: "dirty_early_67b",
    path: ".././crates/rolldown_plugin_vite_resolve/src/vite_resolve_plugin.rs",
  },
  PathCase {
    name: "dirty_late_80b",
    path: "../../crates/rolldown_plugin_vite_resolve/src/vite_resolve_plugin.rs/../index.js",
  },
];

#[cfg(target_family = "windows")]
pub const LEADING_PARENT_SCAN_CASES: &[PathCase] = &[
  PathCase {
    name: "clean_68b",
    path: r"..\..\crates\rolldown_plugin_vite_resolve\src\vite_resolve_plugin.rs",
  },
  PathCase {
    name: "dirty_early_67b",
    path: r"..\.\crates\rolldown_plugin_vite_resolve\src\vite_resolve_plugin.rs",
  },
  PathCase {
    name: "dirty_late_80b",
    path: r"..\..\crates\rolldown_plugin_vite_resolve\src\vite_resolve_plugin.rs\..\index.js",
  },
];

#[cfg(not(target_family = "windows"))]
pub const RELATIVE_CASES: &[RelativeCase] = &[
  RelativeCase {
    name: "module_to_cwd",
    target: "/workspace/rolldown/crates/rolldown/src/module_loader/external_module_task.rs",
    base: "/workspace/rolldown",
  },
  RelativeCase { name: "short_common_prefix", target: "/pkg/assets/index.js", base: "/pkg/chunks" },
  RelativeCase {
    name: "same_directory",
    target: "/workspace/rolldown/crates/rolldown/src/module_loader/external_module_task.rs",
    base: "/workspace/rolldown/crates/rolldown/src/module_loader",
  },
  RelativeCase {
    name: "deep_siblings",
    target: "/workspace/rolldown/crates/rolldown/tests/rolldown/function/chunk_optimization/dynamic_already_loaded_multi_entry/main2.js",
    base: "/workspace/rolldown/crates/rolldown/tests/rolldown/function/chunk_optimization/dynamic_already_loaded_multi_entry/chunks",
  },
  RelativeCase {
    name: "different_subtrees",
    target: "/workspace/rolldown/crates/rolldown_plugin_vite_resolve/src/vite_resolve_plugin.rs",
    base: "/workspace/rolldown/crates/rolldown/src/module_loader",
  },
  RelativeCase {
    name: "dot_slow_path",
    target: "/workspace/rolldown/crates/rolldown/src/./module_loader/../bundle/bundle.rs",
    base: "/workspace/rolldown/crates/rolldown/src/module_loader/..",
  },
  RelativeCase {
    name: "relative_chunk_siblings",
    target: "dist/assets/index-CQFG.js",
    base: "dist/chunks",
  },
  RelativeCase {
    name: "relative_dotted_siblings",
    target: "dist/./assets/index-CQFG.js",
    base: "dist/chunks/../chunks",
  },
  RelativeCase {
    name: "relative_equal_leading_parents",
    target: "../../dist/assets/index-CQFG.js",
    base: "../../dist/chunks",
  },
  RelativeCase {
    name: "relative_unequal_leading_parents",
    target: "../dist/assets/index-CQFG.js",
    base: "../../dist/chunks",
  },
  RelativeCase {
    name: "relative_p99_depth_siblings",
    target: "a/b/c/d/e/f/g/h/assets/index.js",
    base: "a/b/c/d/e/f/g/h/chunks",
  },
  RelativeCase {
    name: "relative_p99_depth_unequal_parents",
    target: "../a/b/c/d/e/f/g/h/assets/index.js",
    base: "../../a/b/c/d/e/f/g/h/chunks",
  },
  RelativeCase { name: "relative_current_directory", target: ".", base: "" },
];

#[cfg(target_family = "windows")]
pub const WINDOWS_RELATIVE_ROOT_CASES: &[RelativeCase] = &[
  RelativeCase {
    name: "forward_slash_absolute",
    target: "C:/workspace/rolldown/crates/rolldown/src/module_loader/external_module_task.rs",
    base: "C:/workspace/rolldown",
  },
  RelativeCase {
    name: "unc_same_share",
    target: r"\\server\share\packages\app\index.js",
    base: r"\\server\share\dist\chunks",
  },
  RelativeCase {
    name: "unc_different_share",
    target: r"\\server\other\packages\app\index.js",
    base: r"\\server\share\dist\chunks",
  },
  RelativeCase {
    name: "verbatim_unc_same_share",
    target: r"\\?\UNC\server\share\packages\app\index.js",
    base: r"\\?\UNC\server\share\dist\chunks",
  },
  RelativeCase {
    name: "verbatim_unc_different_share",
    target: r"\\?\UNC\server\other\packages\app\index.js",
    base: r"\\?\UNC\server\share\dist\chunks",
  },
];

#[cfg(target_family = "windows")]
pub const RELATIVE_CASES: &[RelativeCase] = &[
  RelativeCase {
    name: "module_to_cwd",
    target: r"C:\workspace\rolldown\crates\rolldown\src\module_loader\external_module_task.rs",
    base: r"C:\workspace\rolldown",
  },
  RelativeCase {
    name: "short_common_prefix",
    target: r"C:\pkg\assets\index.js",
    base: r"C:\pkg\chunks",
  },
  RelativeCase {
    name: "same_directory",
    target: r"C:\workspace\rolldown\crates\rolldown\src\module_loader\external_module_task.rs",
    base: r"C:\workspace\rolldown\crates\rolldown\src\module_loader",
  },
  RelativeCase {
    name: "deep_siblings",
    target: r"C:\workspace\rolldown\crates\rolldown\tests\rolldown\function\chunk_optimization\dynamic_already_loaded_multi_entry\main2.js",
    base: r"C:\workspace\rolldown\crates\rolldown\tests\rolldown\function\chunk_optimization\dynamic_already_loaded_multi_entry\chunks",
  },
  RelativeCase {
    name: "different_subtrees",
    target: r"C:\workspace\rolldown\crates\rolldown_plugin_vite_resolve\src\vite_resolve_plugin.rs",
    base: r"C:\workspace\rolldown\crates\rolldown\src\module_loader",
  },
  RelativeCase {
    name: "dot_slow_path",
    target: r"C:\workspace\rolldown\crates\rolldown\src\.\module_loader\..\bundle\bundle.rs",
    base: r"C:\workspace\rolldown\crates\rolldown\src\module_loader\..",
  },
  RelativeCase {
    name: "different_drive",
    target: r"D:\cache\rolldown\index.js",
    base: r"C:\workspace\rolldown\dist\chunks",
  },
  RelativeCase {
    name: "relative_chunk_siblings",
    target: r"dist\assets\index-CQFG.js",
    base: r"dist\chunks",
  },
  RelativeCase {
    name: "relative_dotted_siblings",
    target: r"dist\.\assets\index-CQFG.js",
    base: r"dist\chunks\..\chunks",
  },
  RelativeCase {
    name: "relative_equal_leading_parents",
    target: r"..\..\dist\assets\index-CQFG.js",
    base: r"..\..\dist\chunks",
  },
  RelativeCase {
    name: "relative_unequal_leading_parents",
    target: r"..\dist\assets\index-CQFG.js",
    base: r"..\..\dist\chunks",
  },
  RelativeCase {
    name: "relative_p99_depth_siblings",
    target: r"a\b\c\d\e\f\g\h\assets\index.js",
    base: r"a\b\c\d\e\f\g\h\chunks",
  },
  RelativeCase {
    name: "relative_p99_depth_unequal_parents",
    target: r"..\a\b\c\d\e\f\g\h\assets\index.js",
    base: r"..\..\a\b\c\d\e\f\g\h\chunks",
  },
  RelativeCase { name: "relative_current_directory", target: ".", base: "" },
];
