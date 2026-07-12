#![cfg(all(not(target_family = "windows"), target_arch = "aarch64", target_feature = "neon"))]

use std::path::Path;

use sugar_path::SugarPath;

#[test]
fn absolute_relative_paths_cross_neon_block_boundaries_exactly() {
  for common_prefix_len in [15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256] {
    // Both inputs first differ after `/{shared}/`, whose byte length is the
    // requested common-prefix length.
    let shared = "x".repeat(common_prefix_len - 2);
    let target = format!("/{shared}/target.js");
    let base = format!("/{shared}/chunks");

    assert_eq!(
      Path::new(&target).relative(Path::new(&base)).as_os_str(),
      Path::new("../target.js").as_os_str(),
      "common prefix length {common_prefix_len}",
    );
  }
}
