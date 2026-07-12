//! Tests for deep paths, including the relative slow path beyond its inline component capacity.

#[cfg(target_family = "unix")]
use std::borrow::Cow;
use std::path::Path;
use sugar_path::SugarPath;
mod test_utils;

#[cfg(target_family = "unix")]
#[test]
fn test_normalize_deep_path() {
  // A clean deep path exercises the borrowed fast path.
  let deep_path = "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p";
  let normalized = deep_path.normalize();
  assert!(matches!(&normalized, Cow::Borrowed(_)));
  assert_eq_str!(normalized, deep_path);

  // Dirty deep paths exercise the normalizer and verify the complete result.
  let deep_path_with_dots = "a/b/c/./d/e/f/./g/h/i/./j/k/l/./m/n/o/p";
  let normalized = deep_path_with_dots.normalize();
  assert_eq_str!(normalized, "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p");

  // Test with parent dirs
  let deep_path_with_parents = "a/b/c/../d/e/f/../g/h/i/../j/k/l/../m/n/o/p";
  let normalized = deep_path_with_parents.normalize();
  assert_eq_str!(normalized, "a/b/d/e/g/h/j/k/m/n/o/p");
}

#[cfg(target_family = "unix")]
#[test]
fn test_relative_deep_paths() {
  // Test relative path calculation with more than 8 components
  let base = "/a/b/c/d/e/f/g/h/i/j";
  let target = "/a/b/c/d/e/f/g/k/l/m/n/o/p";

  let relative = target.relative(base);
  assert_eq_str!(relative, "../../../k/l/m/n/o/p");

  // Test with even deeper paths (15+ components)
  let base =
    "/level1/level2/level3/level4/level5/level6/level7/level8/level9/level10/level11/level12";
  let target = "/level1/level2/level3/level4/level5/level6/level7/level8/different9/different10/different11/different12/different13/different14/different15";

  let relative = target.relative(base);
  assert_eq_str!(
    relative,
    "../../../../different9/different10/different11/different12/different13/different14/different15"
  );
}

#[cfg(target_family = "unix")]
#[test]
fn test_relative_dot_normalization_spills_smallvec() {
  // The dot component selects relative_str_slow(), and both normalized paths
  // exceed the inline capacity of SmallVec<[&str; 8]>.
  let base = "/a/b/c/d/e/f/g/h/i/j";
  let target = "/a/b/c/d/e/f/g/h/./i/../x/y";

  assert_eq_str!(target.relative(base), "../../x/y");
}

#[test]
fn test_absolutize_deep_paths() {
  // Test absolutize with deep paths
  #[cfg(target_family = "unix")]
  {
    let base = "/root/level1/level2/level3/level4/level5/level6/level7/level8/level9";
    let relative = "../../../../../../../../../../deep1/deep2/deep3/deep4/deep5";

    let absolute = relative.absolutize_with(base.as_path());
    assert_eq_str!(absolute, "/deep1/deep2/deep3/deep4/deep5");

    // Test with current directory dots in deep path
    let deep_relative = "./sub1/./sub2/./sub3/./sub4/./sub5/./sub6/./sub7/./sub8/./sub9/./sub10";
    let absolute = deep_relative.absolutize_with(base.as_path());
    assert_eq_str!(
      absolute,
      "/root/level1/level2/level3/level4/level5/level6/level7/level8/level9/sub1/sub2/sub3/sub4/sub5/sub6/sub7/sub8/sub9/sub10"
    );
  }

  #[cfg(target_family = "windows")]
  {
    let base = "C:\\root\\level1\\level2\\level3\\level4\\level5\\level6\\level7\\level8\\level9";
    let relative = "..\\..\\..\\..\\..\\..\\..\\..\\..\\..\\deep1\\deep2\\deep3\\deep4\\deep5";

    let absolute = relative.absolutize_with(base.as_path());
    assert_eq_str!(absolute, "C:\\deep1\\deep2\\deep3\\deep4\\deep5");

    // Test with current directory dots in deep path
    let deep_relative =
      ".\\sub1\\.\\sub2\\.\\sub3\\.\\sub4\\.\\sub5\\.\\sub6\\.\\sub7\\.\\sub8\\.\\sub9\\.\\sub10";
    let absolute = deep_relative.absolutize_with(base.as_path());
    assert_eq_str!(
      absolute,
      "C:\\root\\level1\\level2\\level3\\level4\\level5\\level6\\level7\\level8\\level9\\sub1\\sub2\\sub3\\sub4\\sub5\\sub6\\sub7\\sub8\\sub9\\sub10"
    );
  }
}

#[test]
fn test_to_slash_deep_paths() {
  // Test to_slash with deep paths
  #[cfg(target_family = "windows")]
  {
    let deep_path = Path::new("a\\b\\c\\d\\e\\f\\g\\h\\i\\j\\k\\l\\m\\n\\o\\p");
    let slashed = deep_path.to_slash();
    assert_eq!(slashed, "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p");
  }

  #[cfg(target_family = "unix")]
  {
    let deep_path = Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p");
    let slashed = deep_path.to_slash();
    assert_eq!(slashed, "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p");
  }
}

#[test]
fn test_extreme_depth() {
  // Test with 20+ components
  let mut path_parts = Vec::new();
  for i in 0..25 {
    path_parts.push(format!("component{}", i));
  }
  let deep_path = path_parts.join("/");
  let expected =
    if cfg!(target_family = "windows") { deep_path.replace('/', "\\") } else { deep_path.clone() };

  let deep_str = deep_path.as_str();
  let normalized = deep_str.normalize();
  assert_eq_str!(normalized, expected.as_str());

  // Add some dots and test again
  let mut path_with_dots = Vec::new();
  for i in 0..25 {
    path_with_dots.push(format!("component{}", i));
    if i % 3 == 0 {
      path_with_dots.push(".".to_string());
    }
  }
  let deep_path_dots = path_with_dots.join("/");
  let deep_dots_str = deep_path_dots.as_str();
  let normalized = deep_dots_str.normalize();
  assert_eq_str!(normalized, expected.as_str());
}

#[cfg(target_family = "unix")]
#[test]
fn test_normalize_dirty_paths_at_multiple_depths() {
  // Dirty inputs exercise full normalization at several component depths.
  assert_eq_str!("./a/b/c/d/e/f/g/h".normalize(), "a/b/c/d/e/f/g/h");
  assert_eq_str!("a/b/c/d/e/f/g/h/./i".normalize(), "a/b/c/d/e/f/g/h/i");
  assert_eq_str!(
    "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/.".normalize(),
    "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p"
  );

  let complex = "a/./b/../c/d/./e/../f/g/./h/../i/j/./k/../l/m/./n/../o/p/./q/../r";
  let normalized_complex = complex.normalize();
  assert_eq_str!(normalized_complex, "a/c/d/f/g/i/j/l/m/o/p/r");
}

#[cfg(target_family = "windows")]
#[test]
fn test_windows_deep_paths() {
  let deep_win_path = "C:\\level1\\level2\\level3\\level4\\level5\\level6\\level7\\level8\\.\\level9\\level10\\level11\\level12";
  assert_eq_str!(
    deep_win_path.normalize(),
    "C:\\level1\\level2\\level3\\level4\\level5\\level6\\level7\\level8\\level9\\level10\\level11\\level12"
  );

  let unc_path = "\\\\server\\share\\folder1\\folder2\\folder3\\folder4\\folder5\\folder6\\folder7\\folder8\\.\\folder9\\folder10";
  assert_eq_str!(
    unc_path.normalize(),
    "\\\\server\\share\\folder1\\folder2\\folder3\\folder4\\folder5\\folder6\\folder7\\folder8\\folder9\\folder10"
  );
}

#[cfg(target_family = "unix")]
#[test]
fn test_relative_with_common_deep_prefix() {
  // Test paths that share a deep common prefix
  let base = "/shared/path/components/that/are/very/deep/and/long/base/specific/part";
  let target = "/shared/path/components/that/are/very/deep/and/long/target/different/end";

  let relative = target.relative(base);
  assert_eq_str!(relative, "../../../target/different/end");
}
