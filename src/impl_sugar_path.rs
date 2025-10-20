use std::{
  borrow::Cow,
  ops::Deref,
  path::{Component, Path, PathBuf},
};

use crate::{
  SugarPath,
  utils::{IntoCowPath, component_vec_to_path_buf, get_current_dir, to_normalized_components},
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
        let mut components = self.components().collect::<Vec<_>>();
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
      let base_components = base
        .components()
        .filter(|com| {
          matches!(com, Component::Normal(_) | Component::Prefix(_) | Component::RootDir)
        })
        .collect::<Vec<_>>();
      let target_components = target
        .components()
        .filter(|com| {
          matches!(com, Component::Normal(_) | Component::Prefix(_) | Component::RootDir)
        })
        .collect::<Vec<_>>();
      let mut ret = PathBuf::new();
      let longest_len = if base_components.len() > target_components.len() {
        base_components.len()
      } else {
        target_components.len()
      };
      let mut i = 0;
      while i < longest_len {
        let from_component = base_components.get(i);
        let to_component = target_components.get(i);
        // println!("process from: {:?}, to: {:?}", from_component, to_component);
        if cfg!(target_family = "windows")
          && let Some(Component::Normal(from_seg)) = from_component
          && let Some(Component::Normal(to_seg)) = to_component
          && from_seg.eq_ignore_ascii_case(to_seg)
        {
          i += 1;
          continue;
        }
        if from_component != to_component {
          break;
        }
        i += 1;
      }
      let mut from_start = i;
      while from_start < base_components.len() {
        ret.push("..");
        from_start += 1;
      }

      let mut to_start = i;
      while to_start < target_components.len() {
        ret.push(target_components[to_start]);
        to_start += 1;
      }

      ret
    }
  }

  fn to_slash<'a>(&'a self) -> Option<Cow<'a, str>> {
    if std::path::MAIN_SEPARATOR == '/' {
      self.to_str().map(Cow::Borrowed)
    } else {
      self.to_str().map(|s| Cow::Owned(s.replace(std::path::MAIN_SEPARATOR, "/")))
    }
  }

  fn to_slash_lossy<'a>(&'a self) -> Cow<'a, str> {
    if std::path::MAIN_SEPARATOR == '/' {
      self.to_string_lossy()
    } else {
      Cow::Owned(self.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"))
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
