use std::path::Path;

use sugar_path::{SugarPath, SugarPathBuf};

fn assert_matches_composed_api(target: &str, base: &str) {
  let target = Path::new(target);
  let base = Path::new(base);
  let relative = target.relative(base);
  let expected = relative.to_slash().into_owned();
  assert_eq!(relative.into_owned().into_slash(), expected);
}

#[cfg(target_family = "unix")]
#[test]
fn unix_relative_shapes_reuse_the_ordinary_strict_composition() {
  for (target, base) in [
    (
      "/workspace/rolldown/crates/rolldown/src/module_loader/external_module_task.rs",
      "/workspace/rolldown",
    ),
    (
      "/workspace/rolldown/crates/rolldown/src/./module_loader/../bundle/bundle.rs",
      "/workspace/rolldown/crates/rolldown/src/module_loader/..",
    ),
    ("dist/assets/index.js", "dist/chunks"),
    ("dist/./assets/index.js", "dist/chunks/../chunks"),
    ("../../dist/assets/index.js", "../../dist/chunks"),
    ("../dist/assets/index.js", "../../dist/chunks"),
    (".", ""),
  ] {
    assert_matches_composed_api(target, base);
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_relative_prefix_matrix_reuses_the_ordinary_strict_composition() {
  for (target, base) in [
    (r"C:\workspace\rolldown\packages\app\index.js", r"C:\workspace\rolldown\dist\chunks"),
    (r"D:\workspace\rolldown\packages\app\index.js", r"C:\workspace\rolldown\dist\chunks"),
    ("C:/workspace/rolldown/packages/app/index.js", "C:/workspace/rolldown/dist/chunks"),
    (r"C:dist\assets\index.js", r"C:dist\chunks"),
    (r"D:dist\assets\index.js", r"C:dist\chunks"),
    (r"\dist\assets\index.js", r"\dist\chunks"),
    (r"\\server\share\packages\app\index.js", r"\\server\share\dist\chunks"),
    (r"\\server\other\packages\app\index.js", r"\\server\share\dist\chunks"),
    (r"\\other\share\packages\app\index.js", r"\\server\share\dist\chunks"),
    (r"\\?\UNC\server\share\packages\app\index.js", r"\\?\UNC\server\share\dist\chunks"),
    (r"\\?\UNC\server\other\packages\app\index.js", r"\\?\UNC\server\share\dist\chunks"),
    (r"\\?\UNC\other\share\packages\app\index.js", r"\\?\UNC\server\share\dist\chunks"),
    (r"\\?\C:\workspace\packages\app\index.js", r"\\?\C:\workspace\dist\chunks"),
    (r"\\?\D:\workspace\packages\app\index.js", r"\\?\C:\workspace\dist\chunks"),
    (r"\\.\PIPE\rolldown\packages\app\index.js", r"\\.\PIPE\rolldown\dist\chunks"),
    (r"\\.\MAILSLOT\rolldown\packages\app\index.js", r"\\.\PIPE\rolldown\dist\chunks"),
    (r"\\?\GLOBALROOT\rolldown\packages\app\index.js", r"\\?\GLOBALROOT\rolldown\dist\chunks"),
    (r"dist\assets\index.js", r"dist\chunks"),
    (r"dist\.\assets\index.js", r"dist\chunks\..\chunks"),
    (r"..\..\dist\assets\index.js", r"..\..\dist\chunks"),
    (r"..\dist\assets\index.js", r"..\..\dist\chunks"),
    (".", ""),
  ] {
    assert_matches_composed_api(target, base);
  }
}
