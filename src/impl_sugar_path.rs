use std::{borrow::Cow, ops::Deref, path::{Component, Path, PathBuf}};

use crate::{utils::{component_vec_to_path_buf, to_normalized_components}, SugarPath};

impl SugarPath for Path {
  fn normalize(&self) -> PathBuf {
    let mut components = to_normalized_components(self);

    if components.is_empty() {
      return PathBuf::from(".");
    }

    if cfg!(target_family = "windows") {
      if components.len() == 1 && matches!(components[0], Component::Prefix(_)) {
        components.push(Component::CurDir)
      }
    }

    components.into_iter().collect()
  }

  fn absolutize(&self) -> PathBuf {
    self.absolutize_with(std::env::current_dir().unwrap())
  }

  fn absolutize_with(&self, base: impl Into<PathBuf>) -> PathBuf {
    let base: PathBuf = base.into();
    let mut base = if base.is_absolute() { base } else { base.absolutize() };

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
        let mut components = self.components().into_iter().collect::<Vec<_>>();
        components.insert(1, Component::RootDir);
        component_vec_to_path_buf(components).normalize()
      } else {
        base.push(self);
        base.normalize()
      }
    } else {
      base.push(self);
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
        .into_iter()
        .filter(|com| {
          matches!(com, Component::Normal(_) | Component::Prefix(_) | Component::RootDir)
        })
        .collect::<Vec<_>>();
      let target_components = target
        .components()
        .into_iter()
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
        if cfg!(target_family = "windows") {
          if let Some(Component::Normal(from_seg)) = from_component {
            if let Some(Component::Normal(to_seg)) = to_component {
              if from_seg.to_ascii_lowercase() == to_seg.to_ascii_lowercase() {
                i += 1;
                continue;
              }
            }
          }
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

  fn to_slash(&self) -> Option<Cow<str>> {
    if std::path::MAIN_SEPARATOR == '/' {
      self.to_str().map(Cow::Borrowed)
    } else {
      self.to_str().map(|s| Cow::Owned(s.replace(std::path::MAIN_SEPARATOR, "/")))
    }
  }

  fn to_slash_lossy(&self) -> Cow<str> {
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

  fn absolutize_with(&self, base: impl Into<PathBuf>) -> PathBuf {
    self.as_path().absolutize_with(base)
  }

  fn relative(&self, to: impl AsRef<Path>) -> PathBuf {
    self.as_path().relative(to)
  }

  fn to_slash(&self) -> Option<Cow<str>> {
    self.as_path().to_slash()
  }

  fn to_slash_lossy(&self) -> Cow<str> {
    self.as_path().to_slash_lossy()
  }

  fn as_path(&self) -> &Path {
      Path::new(self.deref())
  }
}

fn _test_as_path() {
  let str = "";
  str.as_path();

  let string = String::new();
  string.as_path();

  let ref_string = &string;
  ref_string.as_path();
}