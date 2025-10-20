//! Tests for paths with many components to verify SmallVec spills to heap correctly

use std::path::Path;
use sugar_path::SugarPath;

#[test]
fn test_normalize_deep_path() {
  // Create a path with more than 8 components (SmallVec inline capacity)
  let deep_path = "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p";
  let normalized = deep_path.normalize();
  assert_eq!(normalized, Path::new(deep_path));

  // Test with dots
  let deep_path_with_dots = "a/b/c/./d/e/f/./g/h/i/./j/k/l/./m/n/o/p";
  let normalized = deep_path_with_dots.normalize();
  assert_eq!(normalized, Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p"));

  // Test with parent dirs
  let deep_path_with_parents = "a/b/c/../d/e/f/../g/h/i/../j/k/l/../m/n/o/p";
  let normalized = deep_path_with_parents.normalize();
  assert_eq!(normalized, Path::new("a/b/d/e/g/h/j/k/m/n/o/p"));
}

#[test]
fn test_relative_deep_paths() {
  // Test relative path calculation with more than 8 components
  let base = "/a/b/c/d/e/f/g/h/i/j";
  let target = "/a/b/c/d/e/f/g/k/l/m/n/o/p";

  let relative = target.relative(base);
  assert_eq!(relative, Path::new("../../../k/l/m/n/o/p"));

  // Test with even deeper paths (15+ components)
  let base =
    "/level1/level2/level3/level4/level5/level6/level7/level8/level9/level10/level11/level12";
  let target = "/level1/level2/level3/level4/level5/level6/level7/level8/different9/different10/different11/different12/different13/different14/different15";

  let relative = target.relative(base);
  assert_eq!(
    relative,
    Path::new(
      "../../../../different9/different10/different11/different12/different13/different14/different15"
    )
  );
}

#[test]
fn test_absolutize_deep_paths() {
  // Test absolutize with deep paths
  #[cfg(target_family = "unix")]
  {
    let base = "/root/level1/level2/level3/level4/level5/level6/level7/level8/level9";
    let relative = "../../../../../../../../../../deep1/deep2/deep3/deep4/deep5";

    let absolute = relative.absolutize_with(base);
    assert_eq!(absolute, Path::new("/deep1/deep2/deep3/deep4/deep5"));

    // Test with current directory dots in deep path
    let deep_relative = "./sub1/./sub2/./sub3/./sub4/./sub5/./sub6/./sub7/./sub8/./sub9/./sub10";
    let absolute = deep_relative.absolutize_with(base);
    assert_eq!(
      absolute,
      Path::new(
        "/root/level1/level2/level3/level4/level5/level6/level7/level8/level9/sub1/sub2/sub3/sub4/sub5/sub6/sub7/sub8/sub9/sub10"
      )
    );
  }

  #[cfg(target_family = "windows")]
  {
    let base = "C:\\root\\level1\\level2\\level3\\level4\\level5\\level6\\level7\\level8\\level9";
    let relative = "..\\..\\..\\..\\..\\..\\..\\..\\..\\..\\deep1\\deep2\\deep3\\deep4\\deep5";

    let absolute = relative.absolutize_with(base);
    assert_eq!(absolute, Path::new("C:\\deep1\\deep2\\deep3\\deep4\\deep5"));

    // Test with current directory dots in deep path
    let deep_relative =
      ".\\sub1\\.\\sub2\\.\\sub3\\.\\sub4\\.\\sub5\\.\\sub6\\.\\sub7\\.\\sub8\\.\\sub9\\.\\sub10";
    let absolute = deep_relative.absolutize_with(base);
    assert_eq!(
      absolute,
      Path::new(
        "C:\\root\\level1\\level2\\level3\\level4\\level5\\level6\\level7\\level8\\level9\\sub1\\sub2\\sub3\\sub4\\sub5\\sub6\\sub7\\sub8\\sub9\\sub10"
      )
    );
  }
}

#[test]
fn test_to_slash_deep_paths() {
  // Test to_slash with deep paths
  #[cfg(target_family = "windows")]
  {
    let deep_path = Path::new("a\\b\\c\\d\\e\\f\\g\\h\\i\\j\\k\\l\\m\\n\\o\\p");
    let slashed = deep_path.to_slash().unwrap();
    assert_eq!(slashed, "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p");
  }

  #[cfg(target_family = "unix")]
  {
    let deep_path = Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p");
    let slashed = deep_path.to_slash().unwrap();
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

  // Test normalize
  let normalized = deep_path.as_str().normalize();
  assert_eq!(normalized.components().count(), 25);

  // Add some dots and test again
  let mut path_with_dots = Vec::new();
  for i in 0..25 {
    path_with_dots.push(format!("component{}", i));
    if i % 3 == 0 {
      path_with_dots.push(".".to_string());
    }
  }
  let deep_path_dots = path_with_dots.join("/");
  let normalized = deep_path_dots.as_str().normalize();
  assert_eq!(normalized.components().count(), 25);
}

#[test]
fn test_stress_smallvec_spillover() {
  // Create paths that will definitely spill over SmallVec's inline capacity

  // Test 1: Exactly at boundary (8 components)
  let path8 = "a/b/c/d/e/f/g/h";
  let normalized8 = path8.normalize();
  assert_eq!(normalized8.components().count(), 8);

  // Test 2: Just over boundary (9 components)
  let path9 = "a/b/c/d/e/f/g/h/i";
  let normalized9 = path9.normalize();
  assert_eq!(normalized9.components().count(), 9);

  // Test 3: Well over boundary (16 components)
  let path16 = "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p";
  let normalized16 = path16.normalize();
  assert_eq!(normalized16.components().count(), 16);

  // Test 4: Complex normalization with many components
  let complex = "a/./b/../c/d/./e/../f/g/./h/../i/j/./k/../l/m/./n/../o/p/./q/../r";
  let normalized_complex = complex.normalize();
  // This should normalize to: a/c/d/f/g/i/j/l/m/o/p/r (12 components)
  assert_eq!(normalized_complex, Path::new("a/c/d/f/g/i/j/l/m/o/p/r"));
}

#[test]
fn test_windows_deep_paths() {
  #[cfg(target_family = "windows")]
  {
    // Test Windows-specific deep path handling
    let deep_win_path = "C:\\level1\\level2\\level3\\level4\\level5\\level6\\level7\\level8\\level9\\level10\\level11\\level12";
    let normalized = deep_win_path.normalize();
    assert!(normalized.components().count() > 8);

    // Test UNC paths with many components
    let unc_path = "\\\\server\\share\\folder1\\folder2\\folder3\\folder4\\folder5\\folder6\\folder7\\folder8\\folder9\\folder10";
    let normalized_unc = unc_path.normalize();
    assert!(normalized_unc.components().count() > 8);
  }
}

#[test]
fn test_relative_with_common_deep_prefix() {
  // Test paths that share a deep common prefix
  let base = "/shared/path/components/that/are/very/deep/and/long/base/specific/part";
  let target = "/shared/path/components/that/are/very/deep/and/long/target/different/end";

  let relative = target.relative(base);
  assert_eq!(relative, Path::new("../../../target/different/end"));

  // Verify the path has the expected structure
  let components: Vec<_> = relative.components().collect();
  assert_eq!(components.len(), 6);
}
