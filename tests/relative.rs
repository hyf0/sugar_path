use std::path::Path;

use sugar_path::{SugarPath, SugarPathBuf};
mod test_utils;

#[cfg(target_family = "unix")]
#[test]
fn unix() {
  let cases = [
    ("/var/lib", "/var", ".."),
    ("/var/lib", "/bin", "../../bin"),
    ("/var/lib", "/var/lib", ""),
    ("/var/lib", "/var/apache", "../apache"),
    ("/var/", "/var/lib", "lib"),
    ("/", "/var/lib", "var/lib"),
    ("/foo/test", "/foo/test/bar/package.json", "bar/package.json"),
    ("/Users/a/web/b/test/mails", "/Users/a/web/b", "../.."),
    ("/foo/bar/baz-quux", "/foo/bar/baz", "../baz"),
    ("/foo/bar/baz", "/foo/bar/baz-quux", "../baz-quux"),
    ("/baz-quux", "/baz", "../baz"),
    ("/baz", "/baz-quux", "../baz-quux"),
    ("/page1/page2/foo", "/", "../../.."),
  ];
  cases.into_iter().for_each(|(base, target, expected)| {
    assert_eq_str!(
      Path::new(target).relative(base),
      expected,
      "for input target: {} base: {}",
      target,
      base
    );
  });
}

#[cfg(target_family = "unix")]
#[test]
fn unix_noncanonical_absolute_inputs_are_normalized() {
  for (target, base, expected) in [
    ("/workspace/base/src/", "/workspace/base", "src"),
    ("/workspace/base/src//index.js", "/workspace/base", "src/index.js"),
    ("/workspace/base/src/index.js", "/workspace/base/", "src/index.js"),
    ("/workspace/base/src/index.js", "/workspace//base", "src/index.js"),
  ] {
    assert_eq_str!(Path::new(target).relative(base), expected, "target {target:?}, base {base:?}");
    assert_eq!(Path::new(target).relative(base).into_owned().into_slash(), expected);
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
  let cases = [
    ("c:/blah\\blah", "d:/games", "d:\\games"),
    ("c:/aaaa/bbbb", "c:/aaaa", ".."),
    ("c:/aaaa/bbbb", "c:/cccc", "..\\..\\cccc"),
    ("c:/aaaa/bbbb", "c:/aaaa/bbbb", ""),
    ("c:/aaaa/bbbb", "c:/aaaa/cccc", "..\\cccc"),
    ("c:/aaaa/", "c:/aaaa/cccc", "cccc"),
    ("c:/", "c:\\aaaa\\bbbb", "aaaa\\bbbb"),
    ("c:/aaaa/bbbb", "d:\\", "d:\\"),
    ("c:/AaAa/bbbb", "c:/aaaa/bbbb", ""),
    ("c:/aaaaa/", "c:/aaaa/cccc", "..\\aaaa\\cccc"),
    ("c:/aaaaa/", "d:/aaaa/cccc", "d:\\aaaa\\cccc"),
    ("C:\\foo\\bar\\baz\\quux", "C:\\", "..\\..\\..\\.."),
    ("C:\\foo\\test", "C:\\foo\\test\\bar\\package.json", "bar\\package.json"),
    ("C:\\foo\\bar\\baz-quux", "C:\\foo\\bar\\baz", "..\\baz"),
    ("C:\\foo\\bar\\baz", "C:\\foo\\bar\\baz-quux", "..\\baz-quux"),
    ("\\\\foo\\bar\\baz", "C:\\baz", "C:\\baz"),
    ("C:\\baz", "\\\\foo\\bar\\baz", "\\\\foo\\bar\\baz"),
    ("C:\\baz-quux", "C:\\baz", "..\\baz"),
    ("C:\\baz", "C:\\baz-quux", "..\\baz-quux"),
  ];
  cases.into_iter().for_each(|(base, target, expected)| {
    assert_eq_str!(
      Path::new(target).relative(Path::new(base)),
      expected,
      "for input target: {} base: {}",
      target,
      base
    );
  });
}

#[cfg(target_family = "windows")]
#[test]
fn windows_absolute_remainders_are_normalized() {
  for (target, base, expected) in [
    ("C:/./foo", "C:/", r"foo"),
    ("C:/../foo", "C:/", r"foo"),
    ("C:/foo", "C:/./", r"foo"),
    ("C:/foo", "C:/../bar", r"..\foo"),
    (r"C:\workspace\base\src\", r"C:\workspace\base", r"src"),
    (r"C:\workspace\base\src\\index.js", r"C:\workspace\base", r"src\index.js"),
    (r"C:\workspace\base\src\index.js", r"C:\workspace\base\", r"src\index.js"),
    (r"C:\workspace\base\src\index.js", r"C:\workspace\\base", r"src\index.js"),
  ] {
    assert_eq_str!(Path::new(target).relative(base), expected, "target {target:?}, base {base:?}");
    assert_eq!(
      Path::new(target).relative(base).into_owned().into_slash(),
      expected.replace('\\', "/"),
      "target {target:?}, base {base:?}",
    );
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_unc() {
  let cases = [
    ("\\\\foo\\bar", "\\\\foo\\bar\\baz", "baz"),
    ("\\\\foo\\bar\\baz-quux", "\\\\foo\\bar\\baz", "..\\baz"),
    ("\\\\foo\\baz-quux", "\\\\foo\\baz", "\\\\foo\\baz\\"),
    ("\\\\foo\\bar\\baz", "\\\\foo\\bar\\baz-quux", "..\\baz-quux"),
    ("\\\\foo\\bar\\baz", "\\\\foo\\bar", ".."),
  ];
  cases.into_iter().for_each(|(base, target, expected)| {
    assert_eq_str!(
      Path::new(target).relative(Path::new(base)),
      expected,
      "for input target: {} base: {}",
      target,
      base
    );
  });
}

#[cfg(target_family = "windows")]
fn assert_same_windows_root(base: &str, target: &str, expected: &str) {
  let relative = Path::new(target).relative(Path::new(base));
  assert_eq_str!(relative, expected, "for input target: {} base: {}", target, base);
  assert_eq!(
    Path::new(base).join(&relative).normalize().as_os_str(),
    Path::new(target).normalize().as_os_str(),
    "relative path must rejoin to the normalized target for input target: {} base: {}",
    target,
    base
  );
}

#[cfg(target_family = "windows")]
fn assert_different_windows_roots(base: &str, target: &str) {
  let relative = Path::new(target).relative(Path::new(base));
  assert!(
    relative.is_absolute(),
    "different roots must return an absolute target for input target: {} base: {}",
    target,
    base
  );
  assert_eq!(
    relative.as_os_str(),
    Path::new(target).normalize().as_os_str(),
    "different roots must return the normalized target for input target: {} base: {}",
    target,
    base
  );
}

#[cfg(target_family = "windows")]
fn assert_same_windows_root_ignoring_case(base: &str, target: &str, expected: &str) {
  let relative = Path::new(target).relative(Path::new(base));
  assert_eq_str!(relative, expected, "for input target: {} base: {}", target, base);
  let rejoined = Path::new(base).join(relative).normalize().into_owned();
  let target = Path::new(target).normalize();
  assert!(
    rejoined.to_str().unwrap().eq_ignore_ascii_case(target.to_str().unwrap()),
    "relative path must rejoin case-insensitively for target {target:?} base {base}"
  );
}

#[cfg(target_family = "windows")]
#[test]
fn windows_root_matrix() {
  // Ordinary UNC roots are the server and share together.
  assert_same_windows_root(
    r"\\server\share\dist\chunks",
    r"\\server\share\packages\app\index.js",
    r"..\..\packages\app\index.js",
  );
  assert_same_windows_root_ignoring_case(
    r"\\SERVER\SHARE\dist\chunks",
    r"\\server\share\packages\app\index.js",
    r"..\..\packages\app\index.js",
  );
  assert_different_windows_roots(
    r"\\server\share\dist\chunks",
    r"\\server\other\packages\app\index.js",
  );
  assert_different_windows_roots(
    r"\\server\share\dist\chunks",
    r"\\other\share\packages\app\index.js",
  );

  // Verbatim UNC roots must include the server and share, not just `\\?\UNC`.
  assert_same_windows_root(
    r"\\?\UNC\server\share\dist\chunks",
    r"\\?\UNC\server\share\packages\app\index.js",
    r"..\..\packages\app\index.js",
  );
  assert_same_windows_root_ignoring_case(
    r"\\?\UNC\SERVER\SHARE\dist\chunks",
    r"\\?\UNC\server\share\packages\app\index.js",
    r"..\..\packages\app\index.js",
  );
  assert_different_windows_roots(
    r"\\?\UNC\server\share\dist\chunks",
    r"\\?\UNC\server\other\packages\app\index.js",
  );
  assert_different_windows_roots(
    r"\\?\UNC\server\share\dist\chunks",
    r"\\?\UNC\other\share\packages\app\index.js",
  );

  // The ordinary and verbatim namespaces are distinct even when their server
  // and share names match.
  assert_different_windows_roots(
    r"\\server\share\dist\chunks",
    r"\\?\UNC\server\share\packages\app\index.js",
  );
  assert_different_windows_roots(
    r"\\?\UNC\server\share\dist\chunks",
    r"\\server\share\packages\app\index.js",
  );

  // Drive, verbatim drive, and device namespace prefixes each define their
  // own roots.
  assert_same_windows_root(
    r"C:\workspace\rolldown\dist\chunks",
    r"C:\workspace\rolldown\packages\app\index.js",
    r"..\..\packages\app\index.js",
  );
  assert_same_windows_root_ignoring_case(
    r"c:\workspace\rolldown\dist\chunks",
    r"C:\workspace\rolldown\packages\app\index.js",
    r"..\..\packages\app\index.js",
  );
  assert_different_windows_roots(
    r"C:\workspace\rolldown\dist\chunks",
    r"D:\workspace\rolldown\packages\app\index.js",
  );
  assert_same_windows_root(
    r"\\?\C:\workspace\rolldown\dist\chunks",
    r"\\?\C:\workspace\rolldown\packages\app\index.js",
    r"..\..\packages\app\index.js",
  );
  assert_same_windows_root_ignoring_case(
    r"\\?\c:\workspace\rolldown\dist\chunks",
    r"\\?\C:\workspace\rolldown\packages\app\index.js",
    r"..\..\packages\app\index.js",
  );
  assert_different_windows_roots(
    r"\\?\C:\workspace\rolldown\dist\chunks",
    r"\\?\D:\workspace\rolldown\packages\app\index.js",
  );
  assert_different_windows_roots(
    r"C:\workspace\rolldown\dist\chunks",
    r"\\?\C:\workspace\rolldown\packages\app\index.js",
  );
  assert_same_windows_root(
    r"\\.\PIPE\rolldown\dist\chunks",
    r"\\.\PIPE\rolldown\packages\app\index.js",
    r"..\..\packages\app\index.js",
  );
  assert_different_windows_roots(
    r"\\.\PIPE\rolldown\dist\chunks",
    r"\\.\MAILSLOT\rolldown\packages\app\index.js",
  );
}

#[cfg(target_family = "windows")]
#[test]
fn windows_different_roots_strip_a_non_root_target_trailing_separator() {
  assert_eq_str!(Path::new(r"d:\target\").relative(r"C:\base"), r"d:\target");
  assert_eq_str!(
    Path::new(r"\\server\other\target\").relative(r"\\server\share\base"),
    r"\\server\other\target",
  );
}

#[cfg(target_family = "windows")]
#[test]
fn windows_unrepresentable_relative_components_return_the_absolute_target() {
  for (base, target) in [
    (r"\\?\C:\base", r"\\?\C:\base\foo/bar"),
    (r"\\?\UNC\server\share\base", r"\\?\UNC\server\share\base\foo/bar"),
    (r"\\?\Volume{abc}\base", r"\\?\Volume{abc}\base\foo/bar"),
    (r"C:\base", r"C:\base\C:foo"),
  ] {
    let relative = Path::new(target).relative(base);
    assert!(relative.is_absolute(), "target {target:?}, base {base:?}");
    assert_eq_str!(relative, target, "target {target:?}, base {base:?}");
  }

  // A literal slash in the base is one verbatim component, so the component
  // fallback emits one parent rather than splitting it into two components.
  assert_eq_str!(Path::new(r"\\?\C:\base\other").relative(r"\\?\C:\base\foo/bar"), r"..\other",);

  assert_eq_str!(Path::new(r"C:\base\dir\C:foo").relative(r"C:\base"), r"dir\C:foo");
  assert_eq_str!(Path::new(r"C:\base\C:foo").relative(r"C:\base\other"), r"..\C:foo");

  for (base, target, expected) in
    [(r"dist", r"dist\C:foo", r"C:\cwd\dist\C:foo"), (r"C:.", r"C:C:foo", r"C:\cwd\C:foo")]
  {
    let relative = Path::new(target).relative_with(base, r"C:\cwd");
    assert!(relative.is_absolute(), "target {target:?}, base {base:?}");
    assert_eq_str!(relative, expected, "target {target:?}, base {base:?}");
  }

  assert_eq_str!(Path::new(r"dir\C:foo").relative_with("", "not/absolute"), r"dir\C:foo",);
  assert_eq_str!(Path::new(r".\C:foo").relative_with("other", "not/absolute"), r"..\C:foo",);
}

#[cfg(target_family = "windows")]
#[test]
fn windows_relative_only_ignores_ascii_case() {
  assert_eq_str!(Path::new(r"c:\i̇\b").relative(r"C:\İ\a"), r"..\..\i̇\b");
}
