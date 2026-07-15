use std::borrow::Cow;
use sugar_path::SugarPath;
mod test_utils;

#[cfg(target_family = "unix")]
#[test]
fn unix() {
  assert_eq_str!(p!("/foo/../../../bar").normalize(), "/bar");
  assert_eq_str!(p!("a//b//../b").normalize(), "a/b");
  assert_eq_str!(p!("a//b//./c").normalize(), "a/b/c");
  assert_eq_str!(p!("a//b//.").normalize(), "a/b");
  assert_eq_str!(p!("/a/b/c/../../../x/y/z").normalize(), "/x/y/z");
  assert_eq_str!(p!("///..//./foo/.//bar").normalize(), "/foo/bar");
  assert_eq_str!(p!("bar/foo../../").normalize(), "bar/");
  assert_eq_str!(p!("bar/foo../..").normalize(), "bar");
  assert_eq_str!(p!("bar/foo../../baz").normalize(), "bar/baz");
  assert_eq_str!(p!("bar/foo../").normalize(), "bar/foo../");
  assert_eq_str!(p!("bar/foo..").normalize(), "bar/foo..");
  assert_eq_str!(p!("../foo../../../bar").normalize(), "../../bar");
  assert_eq_str!(p!("../.../.././.../../../bar").normalize(), "../../bar");
  assert_eq_str!(p!("../../../foo/../../../bar").normalize(), "../../../../../bar");
  assert_eq_str!(p!("../../../foo/../../../bar/../../").normalize(), "../../../../../../");
  assert_eq_str!(p!("../foobar/barfoo/foo/../../../bar/../../").normalize(), "../../");
  assert_eq_str!(p!("../.../../foobar/../../../bar/../../baz").normalize(), "../../../../baz");
  assert_eq_str!(p!("foo/bar\\baz").normalize(), "foo/bar\\baz");
  assert_eq_str!(p!("/a/b/c/../../../").normalize(), "/");
  assert_eq_str!(p!("a/b/c/../../../").normalize(), "./");
  assert_eq_str!(p!("a/b/c/../../..").normalize(), ".");

  assert_eq_str!(p!("").normalize(), ".");
}

#[cfg(target_family = "windows")]
#[test]
fn windows() {
  assert_eq_str!(p!("").normalize(), ".");
  assert_eq_str!(p!("./fixtures///b/../b/c.js").normalize(), "fixtures\\b\\c.js");
  assert_eq_str!(p!("/foo/../../../bar").normalize(), "\\bar");
  assert_eq_str!(p!("a//b//../b").normalize(), "a\\b");
  assert_eq_str!(p!("a//b//./c").normalize(), "a\\b\\c");
  assert_eq_str!(p!("//server/share/dir/file.ext").normalize(), "\\\\server\\share\\dir\\file.ext");
  assert_eq_str!(p!("/a/b/c/../../../x/y/z").normalize(), "\\x\\y\\z");
  assert_eq_str!(p!("C:").normalize(), "C:.");
  assert_eq_str!(p!("C:/").normalize(), "C:\\");
  assert_eq_str!(p!("c:/ignore").normalize(), "c:\\ignore");
  assert_eq_str!(p!("C:../a").normalize(), "C:..\\a");
  assert_eq_str!(p!("c:/../a").normalize(), "c:\\a");
  assert_eq_str!(p!("C:..\\..\\abc\\..\\def").normalize(), "C:..\\..\\def");
  assert_eq_str!(p!("C:\\..\\..\\abc\\..\\def").normalize(), "C:\\def");
  assert_eq_str!(p!("C:\\.").normalize(), "C:\\");

  assert_eq_str!(p!("file:stream").normalize(), "file:stream");
  assert_eq_str!(p!("bar\\foo..\\..\\").normalize(), "bar\\");
  assert_eq_str!(p!("bar\\foo..\\..").normalize(), "bar");
  assert_eq_str!(p!("bar\\foo..\\..\\baz").normalize(), "bar\\baz");
  assert_eq_str!(p!("bar\\foo..\\").normalize(), "bar\\foo..\\");
  assert_eq_str!(p!("..\\foo..\\..\\..\\bar").normalize(), "..\\..\\bar");
  assert_eq_str!(p!("..\\...\\..\\.\\...\\..\\..\\bar").normalize(), "..\\..\\bar");
  assert_eq_str!(p!("../../../foo/../../../bar").normalize(), "..\\..\\..\\..\\..\\bar");
  assert_eq_str!(p!("../../../foo/../../../bar/../../").normalize(), "..\\..\\..\\..\\..\\..\\");
  assert_eq_str!(p!("../foobar/barfoo/foo/../../../bar/../../").normalize(), "..\\..\\");
  assert_eq_str!(p!("../.../../foobar/../../../bar/../../baz").normalize(), "..\\..\\..\\..\\baz");
  assert_eq_str!(p!("foo/bar\\baz").normalize(), "foo\\bar\\baz");
}

#[cfg(target_family = "windows")]
#[test]
fn windows_prefix_only_paths_preserve_their_prefix_semantics() {
  use std::path::{Component, Prefix};

  let cases = [
    (r"C:", r"C:."),
    (r"c:", r"c:."),
    (r"\\?\C:", r"\\?\C:"),
    (r"\\?\c:", r"\\?\c:"),
    (r"\\server\share", r"\\server\share\"),
    (r"\\?\UNC\server\share", r"\\?\UNC\server\share"),
    (r"\\?\UNC\server\share\.", r"\\?\UNC\server\share"),
    (r"\\?\UNC\server\share\foo\..", r"\\?\UNC\server\share"),
    (r"\\?\UNC\server\share\foo\..\", r"\\?\UNC\server\share\"),
    (r"\\.\PIPE", r"\\.\PIPE"),
    (r"\\.\PIPE\.", r"\\.\PIPE"),
    (r"\\.\PIPE\foo\..", r"\\.\PIPE"),
    (r"\\.\PIPE\foo\..\", r"\\.\PIPE\"),
    (r"\\?\Volume{abc}", r"\\?\Volume{abc}"),
    (r"\\?\Volume{abc}\.", r"\\?\Volume{abc}"),
    (r"\\?\Volume{abc}\foo\..", r"\\?\Volume{abc}"),
    (r"\\?\Volume{abc}\foo\..\", r"\\?\Volume{abc}\"),
  ];

  for (input, expected) in cases {
    let normalized = p!(input).normalize();
    assert_eq_str!(normalized, expected);
    assert_eq_str!(normalized.normalize(), expected);
  }

  macro_rules! assert_prefix_semantics {
    ($input:expr, $kind:pat, $rooted:expr, $has_root_component:expr) => {{
      let path = p!($input);
      let components = path.components().collect::<Vec<_>>();
      assert_eq!(path.has_root(), $rooted, "unexpected has_root for {:?}", path);
      assert_eq!(path.is_absolute(), $rooted, "unexpected is_absolute for {:?}", path);
      assert!(
        matches!(components.first(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), $kind)),
        "unexpected prefix for {:?}: {:?}",
        path,
        components,
      );
      assert_eq!(
        matches!(components.get(1), Some(Component::RootDir)),
        $has_root_component,
        "unexpected RootDir component for {:?}: {:?}",
        path,
        components,
      );
      assert_eq!(components.len(), if $has_root_component { 2 } else { 1 });
    }};
  }

  assert_prefix_semantics!(r"C:", Prefix::Disk(b'C'), false, false);
  assert_prefix_semantics!(r"\\?\C:", Prefix::VerbatimDisk(b'C'), true, false);
  assert_prefix_semantics!(r"\\server\share", Prefix::UNC(_, _), true, true);
  assert_prefix_semantics!(r"\\?\UNC\server\share", Prefix::VerbatimUNC(_, _), true, false);
  assert_prefix_semantics!(r"\\.\PIPE", Prefix::DeviceNS(_), true, true);
  assert_prefix_semantics!(r"\\?\Volume{abc}", Prefix::Verbatim(_), true, false);

  // Rust considers all verbatim prefixes rooted and absolute, but `\\?\C:`
  // itself has no explicit RootDir component and normalization must not add one.
  let verbatim_disk = p!(r"\\?\C:");
  let normalized = verbatim_disk.normalize();
  assert_eq!(normalized, verbatim_disk);
  assert!(!normalized.components().any(|component| component == Component::RootDir));
}

#[cfg(target_family = "windows")]
#[test]
fn windows_verbatim_paths_treat_forward_slashes_as_literal_characters() {
  for path in [r"\\?\C:\foo/", r"\\?\UNC\server\share\foo/", r"\\?\Volume{abc}\foo/"] {
    let normalized = p!(path).normalize();
    assert_eq_str!(normalized, path);
    assert_eq_str!(normalized.normalize(), path);
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_normalization_does_not_reparse_normal_components_as_prefixes() {
  use std::path::Component;

  for (path, expected) in
    [(r"dir\..\C:foo", r".\C:foo"), (r".\C:foo", r".\C:foo"), (r"dir\..\C:", r".\C:")]
  {
    let normalized = p!(path).normalize();
    assert_eq_str!(normalized, expected);
    assert_eq_str!(normalized.normalize(), expected);
    assert!(matches!(
      normalized.components().collect::<Vec<_>>().as_slice(),
      [Component::CurDir, Component::Normal(_)]
    ));
  }
}

#[cfg(target_family = "unix")]
#[test]
fn unix_clean_paths_return_borrowed_cow() {
  for (path, expected) in [
    ("", "."),
    (".", "."),
    ("/", "/"),
    ("/usr/local/bin", "/usr/local/bin"),
    ("foo/bar/baz", "foo/bar/baz"),
    ("foo/bar/", "foo/bar/"),
    ("foo", "foo"),
    ("/home/user/file.txt", "/home/user/file.txt"),
    ("...", "..."),
    (".foo", ".foo"),
  ] {
    let normalized = p!(path).normalize();
    assert!(
      matches!(normalized, Cow::Borrowed(_)),
      "expected borrowed Cow for clean path {:?}",
      path,
    );
    assert_eq_str!(normalized, expected);
  }
}

#[cfg(target_family = "unix")]
#[test]
fn unix_clean_current_directory_spellings_return_borrowed_cow() {
  for (path, expected) in [("", "."), (".", "."), ("./", "./")] {
    let normalized = p!(path).normalize();
    assert!(matches!(normalized, Cow::Borrowed(_)), "expected borrowed Cow for {path:?}");
    assert_eq!(normalized.as_os_str(), p!(expected).as_os_str());
  }
}

#[cfg(target_family = "unix")]
#[test]
fn unix_dirty_paths_return_owned_cow() {
  for (path, expected) in [
    ("./foo", "foo"),
    ("foo/../bar", "bar"),
    ("foo//bar", "foo/bar"),
    ("foo/./bar", "foo/bar"),
    ("/foo/../bar", "/bar"),
  ] {
    let normalized = p!(path).normalize();
    assert!(matches!(normalized, Cow::Owned(_)), "expected owned Cow for dirty path {:?}", path,);
    assert_eq_str!(normalized, expected);
  }
}

#[cfg(target_family = "unix")]
#[test]
fn unix_canonical_leading_parent_paths_return_borrowed_cow() {
  for path in ["..", "../", "../foo", "../foo/", "../../chunks/shared.js"] {
    let input = p!(path);
    let normalized = input.normalize();
    let Cow::Borrowed(borrowed) = normalized else {
      panic!("expected borrowed Cow for canonical leading-parent path {path:?}");
    };
    assert_eq!(borrowed.as_os_str(), input.as_os_str());
  }
}

#[cfg(target_family = "unix")]
#[test]
fn unix_dirty_leading_parent_paths_return_owned_cow() {
  for (path, expected) in [
    (".././foo", "../foo"),
    ("../foo/..", ".."),
    ("../foo//bar", "../foo/bar"),
    ("../../foo/../bar", "../../bar"),
  ] {
    let normalized = p!(path).normalize();
    assert!(matches!(normalized, Cow::Owned(_)), "expected owned Cow for dirty path {path:?}");
    assert_eq!(normalized.as_os_str(), p!(expected).as_os_str());
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_clean_paths_return_borrowed_cow() {
  for (path, expected) in [
    (r"C:\foo\bar", r"C:\foo\bar"),
    (r"c:\foo", r"c:\foo"),
    (r"C:\", r"C:\"),
    (r"\foo\bar", r"\foo\bar"),
    (r"foo\bar\baz", r"foo\bar\baz"),
    (r"foo\bar\", r"foo\bar\"),
    (r"C:foo", r"C:foo"),
    ("foo", "foo"),
    (r"\", r"\"),
    ("...", "..."),
    (".foo", ".foo"),
    ("", "."),
    (".", "."),
  ] {
    let normalized = p!(path).normalize();
    assert!(
      matches!(normalized, Cow::Borrowed(_)),
      "expected borrowed Cow for clean path {:?}",
      path,
    );
    assert_eq_str!(normalized, expected);
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_clean_current_directory_spellings_return_borrowed_cow() {
  for (path, expected) in
    [("", "."), (".", "."), (r".\", r".\"), ("C:.", "C:."), (r"C:.\", r"C:.\")]
  {
    let normalized = p!(path).normalize();
    assert!(matches!(normalized, Cow::Borrowed(_)), "expected borrowed Cow for {path:?}");
    assert_eq!(normalized.as_os_str(), p!(expected).as_os_str());
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_dirty_current_directory_spellings_are_canonicalized() {
  for (path, expected) in [(r".\\", r".\"), (r".\.", "."), (r"C:.\.", "C:."), (r"C:.\..", "C:..")] {
    let normalized = p!(path).normalize();
    assert_eq!(normalized.as_os_str(), p!(expected).as_os_str());
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_dirty_paths_return_owned_cow() {
  for (path, expected) in [
    ("C:", "C:."),
    ("C:/foo", r"C:\foo"),
    (r"foo\\bar", r"foo\bar"),
    (r"foo\..\bar", "bar"),
    (r"C:.\file", r"C:file"),
    (r"\\server\share\dir", r"\\server\share\dir"),
    (r".\foo", "foo"),
    (r"foo\.\bar", r"foo\bar"),
  ] {
    let normalized = p!(path).normalize();
    assert!(matches!(normalized, Cow::Owned(_)), "expected owned Cow for dirty path {:?}", path,);
    assert_eq_str!(normalized, expected);
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_canonical_leading_parent_paths_return_borrowed_cow() {
  for path in [
    r"..",
    r"..\",
    r"..\foo",
    r"..\foo\",
    r"..\..\chunks\shared.js",
    r"C:..\..",
    r"c:..\..\chunks\shared.js",
  ] {
    let input = p!(path);
    let normalized = input.normalize();
    let Cow::Borrowed(borrowed) = normalized else {
      panic!("expected borrowed Cow for canonical leading-parent path {path:?}");
    };
    assert_eq!(borrowed.as_os_str(), input.as_os_str());
  }
}

#[cfg(target_family = "windows")]
#[test]
fn windows_dirty_leading_parent_paths_return_owned_cow() {
  for (path, expected) in [
    (r"..\.\foo", r"..\foo"),
    (r"..\foo\..", r".."),
    (r"..\foo\\bar", r"..\foo\bar"),
    (r"..\..\foo\..\bar", r"..\..\bar"),
    (r"C:..\foo\..", r"C:.."),
  ] {
    let normalized = p!(path).normalize();
    assert!(matches!(normalized, Cow::Owned(_)), "expected owned Cow for dirty path {path:?}");
    assert_eq!(normalized.as_os_str(), p!(expected).as_os_str());
  }
}
