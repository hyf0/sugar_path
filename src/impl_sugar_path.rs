use std::{
  borrow::Cow,
  ops::Deref,
  path::{Component, Path, PathBuf},
};

use smallvec::SmallVec;

use crate::{
  SugarPath,
  utils::{
    ComponentVec, IntoCowPath, component_vec_to_path_buf, get_current_dir, to_normalized_components,
  },
};

impl SugarPath for Path {
  fn normalize(&self) -> PathBuf {
    let mut components = to_normalized_components(self);

    if components.is_empty() {
      return PathBuf::from(".");
    }

    if cfg!(target_family = "windows")
      && components.len() == 1
      && matches!(components[0], Component::Prefix(_))
    {
      components.push(Component::CurDir)
    }

    components.into_iter().collect()
  }

  fn absolutize(&self) -> PathBuf {
    self.absolutize_with(get_current_dir())
  }

  // Using `Cow` is on purpose.
  // - Users could choose to pass a reference or an owned value depending on their use case.
  // - If we accept `PathBuf` only, it may cause unnecessary allocations on case that `self` is already absolute.
  // - If we accept `&Path` only, it may cause unnecessary cloning that users already have an owned value.
  fn absolutize_with<'a>(&self, base: impl IntoCowPath<'a>) -> PathBuf {
    let base: Cow<'a, Path> = base.into_cow_path();
    let mut base = if base.is_absolute() { base } else { Cow::Owned(base.absolutize()) };

    if self.is_absolute() {
      self.normalize()
    } else if cfg!(target_family = "windows") {
      // Consider c:
      let mut components = self.components();
      if matches!(components.next(), Some(Component::Prefix(_)))
        && !matches!(components.next(), Some(Component::RootDir))
      {
        // TODO: Windows has the concept of drive-specific current working
        // directories. If we've resolved a drive letter but not yet an
        // absolute path, get cwd for that drive, or the process cwd if
        // the drive cwd is not available. We're sure the device is not
        // a UNC path at this points, because UNC paths are always absolute.
        let mut components: ComponentVec = self.components().collect();
        components.insert(1, Component::RootDir);
        component_vec_to_path_buf(components).normalize()
      } else {
        base.to_mut().push(self);
        base.normalize()
      }
    } else {
      base.to_mut().push(self);
      base.normalize()
    }
  }

  fn relative(&self, to: impl AsRef<Path>) -> PathBuf {
    let base = to.as_ref().absolutize();
    let target = self.absolutize();
    if base == target {
      PathBuf::new()
    } else {
      // Filter components inline
      let filter_fn = |com: &Component| {
        matches!(com, Component::Normal(_) | Component::Prefix(_) | Component::RootDir)
      };

      // Collect components using SmallVec to avoid heap allocation for typical paths
      let base_components: ComponentVec = base.components().filter(filter_fn).collect();
      let target_components: ComponentVec = target.components().filter(filter_fn).collect();

      // Find common prefix length
      let common_len = base_components
        .iter()
        .zip(target_components.iter())
        .take_while(|(from, to)| {
          // Handle Windows case-insensitive comparison
          if cfg!(target_family = "windows")
            && let (Component::Normal(from_seg), Component::Normal(to_seg)) = (from, to)
          {
            return from_seg.eq_ignore_ascii_case(to_seg);
          }
          from == to
        })
        .count();

      // Build the result path without repeated PathBuf::push allocations
      let up_len = base_components.len().saturating_sub(common_len);
      let down_len = target_components.len().saturating_sub(common_len);
      let mut components: ComponentVec<'_> = SmallVec::new();
      components.reserve(up_len + down_len);

      for _ in 0..up_len {
        components.push(Component::ParentDir);
      }

      components.extend(target_components[common_len..].iter().cloned());

      component_vec_to_path_buf(components)
    }
  }

  fn to_slash<'a>(&'a self) -> Option<Cow<'a, str>> {
    if std::path::MAIN_SEPARATOR == '/' {
      self.to_str().map(Cow::Borrowed)
    } else {
      self.to_str().map(|s| match replace_main_separator(s) {
        Some(replaced) => Cow::Owned(replaced),
        None => Cow::Borrowed(s),
      })
    }
  }

  fn to_slash_lossy<'a>(&'a self) -> Cow<'a, str> {
    if std::path::MAIN_SEPARATOR == '/' {
      self.to_string_lossy()
    } else {
      match self.to_string_lossy() {
        Cow::Borrowed(s) => match replace_main_separator(s) {
          Some(replaced) => Cow::Owned(replaced),
          None => Cow::Borrowed(s),
        },
        Cow::Owned(owned) => match replace_main_separator(&owned) {
          Some(replaced) => Cow::Owned(replaced),
          None => Cow::Owned(owned),
        },
      }
    }
  }

  fn as_path(&self) -> &Path {
    self
  }
}

impl<T: Deref<Target = str>> SugarPath for T {
  fn normalize(&self) -> PathBuf {
    self.as_path().normalize()
  }

  fn absolutize(&self) -> PathBuf {
    self.as_path().absolutize()
  }

  fn absolutize_with<'a>(&self, base: impl IntoCowPath<'a>) -> PathBuf {
    self.as_path().absolutize_with(base)
  }

  fn relative(&self, to: impl AsRef<Path>) -> PathBuf {
    self.as_path().relative(to)
  }

  fn to_slash<'a>(&'a self) -> Option<Cow<'a, str>> {
    self.as_path().to_slash()
  }

  fn to_slash_lossy<'a>(&'a self) -> Cow<'a, str> {
    self.as_path().to_slash_lossy()
  }

  fn as_path(&self) -> &Path {
    Path::new(self.deref())
  }
}

fn replace_main_separator(input: &str) -> Option<String> {
  let sep = std::path::MAIN_SEPARATOR;
  let mut replaced: Option<String> = None;
  let mut segment_start = 0;

  for (idx, ch) in input.char_indices() {
    if ch == sep {
      let buf = replaced.get_or_insert_with(|| String::with_capacity(input.len()));
      buf.push_str(&input[segment_start..idx]);
      buf.push('/');
      segment_start = idx + ch.len_utf8();
    }
  }

  if let Some(mut buf) = replaced {
    buf.push_str(&input[segment_start..]);
    Some(buf)
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use std::{borrow::Cow, path::Path, path::PathBuf};

  use super::SugarPath;

  #[test]
  fn _test_as_path() {
    let str = "";
    str.as_path();

    let string = String::new();
    string.as_path();

    let ref_string = &string;
    ref_string.as_path();
  }

  #[test]
  fn _test_absolutize_with() {
    let tmp = "";

    let str = "";
    tmp.absolutize_with(str);

    let string = String::new();
    tmp.absolutize_with(string);

    let ref_string = &String::new();
    tmp.absolutize_with(ref_string);

    let path = Path::new("");
    tmp.absolutize_with(path);

    let path_buf = PathBuf::new();
    tmp.absolutize_with(path_buf);

    let cow_path = Cow::Borrowed(Path::new(""));
    tmp.absolutize_with(cow_path);

    let cow_str = Cow::Borrowed("");
    tmp.absolutize_with(cow_str);
  }

  #[test]
  fn normalize() {
    assert_eq!(Path::new("/foo/../../../bar").normalize(), Path::new("/bar"));
    assert_eq!(Path::new("a//b//../b").normalize(), Path::new("a/b"));
    assert_eq!(Path::new("/foo/../../../bar").normalize(), Path::new("/bar"));
    assert_eq!(Path::new("a//b//./c").normalize(), Path::new("a/b/c"));
    assert_eq!(Path::new("a//b//.").normalize(), Path::new("a/b"));
    assert_eq!(Path::new("/a/b/c/../../../x/y/z").normalize(), Path::new("/x/y/z"));
    assert_eq!(Path::new("///..//./foo/.//bar").normalize(), Path::new("/foo/bar"));
    assert_eq!(Path::new("bar/foo../../").normalize(), Path::new("bar/"));
    assert_eq!(Path::new("bar/foo../..").normalize(), Path::new("bar"));
    assert_eq!(Path::new("bar/foo../../baz").normalize(), Path::new("bar/baz"));
    assert_eq!(Path::new("bar/foo../").normalize(), Path::new("bar/foo../"));
    assert_eq!(Path::new("bar/foo..").normalize(), Path::new("bar/foo.."));
    assert_eq!(Path::new("../foo../../../bar").normalize(), Path::new("../../bar"));
    assert_eq!(Path::new("../foo../../../bar").normalize(), Path::new("../../bar"));
    assert_eq!(Path::new("../.../.././.../../../bar").normalize(), Path::new("../../bar"));
    assert_eq!(Path::new("../.../.././.../../../bar").normalize(), Path::new("../../bar"));
    assert_eq!(Path::new("../../../foo/../../../bar").normalize(), Path::new("../../../../../bar"));
    assert_eq!(
      Path::new("../../../foo/../../../bar/../../").normalize(),
      Path::new("../../../../../../")
    );
    assert_eq!(
      Path::new("../foobar/barfoo/foo/../../../bar/../../").normalize(),
      Path::new("../../")
    );
    assert_eq!(
      Path::new("../.../../foobar/../../../bar/../../baz").normalize(),
      Path::new("../../../../baz")
    );
    assert_eq!(Path::new("foo/bar\\baz").normalize(), Path::new("foo/bar\\baz"));
    assert_eq!(Path::new("/a/b/c/../../../").normalize(), Path::new("/"));
    assert_eq!(Path::new("a/b/c/../../../").normalize(), Path::new("."));
    assert_eq!(Path::new("a/b/c/../../..").normalize(), Path::new("."));

    assert_eq!(Path::new("").normalize(), Path::new("."));
  }
}
