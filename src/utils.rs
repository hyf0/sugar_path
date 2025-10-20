use smallvec::SmallVec;
use std::{
  borrow::Cow,
  path::{Component, Path, PathBuf},
  sync::OnceLock,
};

static CURRENT_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn get_current_dir() -> Cow<'static, Path> {
  if cfg!(feature = "cached_current_dir") {
    let s: &'static Path = CURRENT_DIR.get_or_init(|| std::env::current_dir().unwrap());
    Cow::Borrowed(s)
  } else {
    Cow::Owned(std::env::current_dir().unwrap())
  }
}

pub trait IntoCowPath<'a> {
  fn into_cow_path(self) -> Cow<'a, Path>;
}

impl<'a> IntoCowPath<'a> for &'a Path {
  fn into_cow_path(self) -> Cow<'a, Path> {
    Cow::Borrowed(self)
  }
}

impl<'a> IntoCowPath<'a> for PathBuf {
  fn into_cow_path(self) -> Cow<'a, Path> {
    Cow::Owned(self)
  }
}

impl<'a> IntoCowPath<'a> for &'a PathBuf {
  fn into_cow_path(self) -> Cow<'a, Path> {
    Cow::Borrowed(self.as_path())
  }
}

impl<'a> IntoCowPath<'a> for &'a str {
  fn into_cow_path(self) -> Cow<'a, Path> {
    Cow::Borrowed(Path::new(self))
  }
}

impl<'a> IntoCowPath<'a> for String {
  fn into_cow_path(self) -> Cow<'a, Path> {
    Cow::Owned(PathBuf::from(self))
  }
}

impl<'a> IntoCowPath<'a> for &'a String {
  fn into_cow_path(self) -> Cow<'a, Path> {
    Cow::Borrowed(Path::new(self))
  }
}

impl<'a> IntoCowPath<'a> for Cow<'a, Path> {
  fn into_cow_path(self) -> Cow<'a, Path> {
    match self {
      Cow::Borrowed(path) => Cow::Borrowed(path),
      Cow::Owned(path) => Cow::Owned(path),
    }
  }
}

impl<'a> IntoCowPath<'a> for Cow<'a, str> {
  fn into_cow_path(self) -> Cow<'a, Path> {
    match self {
      Cow::Borrowed(s) => s.into_cow_path(),
      Cow::Owned(s) => s.into_cow_path(),
    }
  }
}

// Type alias for SmallVec with stack capacity of 8 (typical path depth)
pub type ComponentVec<'a> = SmallVec<[Component<'a>; 8]>;

#[inline]
pub fn component_vec_to_path_buf(components: ComponentVec) -> PathBuf {
  components.into_iter().collect()
}

pub fn to_normalized_components<'a>(path: &'a Path) -> ComponentVec<'a> {
  let mut components = path.components().peekable();
  let mut ret = SmallVec::with_capacity(components.size_hint().1.unwrap_or(0));
  if let Some(c @ Component::Prefix(..)) = components.peek() {
    ret.push(*c);
    components.next();
  };

  for component in components {
    match component {
      Component::Prefix(..) => unreachable!("Unexpected prefix for {:?}", path.display()),
      Component::RootDir => {
        ret.push(component);
      }
      Component::CurDir => {
        // ignore
      }
      c @ Component::ParentDir => {
        // So we hit a `..` here. If the previous path segment looks like
        // - `c:`
        // - `c:../..`
        // - `../..`
        // - ``
        // We should preserve the `..`

        let need_to_preserve =
          matches!(ret.last(), None | Some(Component::Prefix(_)) | Some(Component::ParentDir));
        if need_to_preserve {
          ret.push(c);
        } else {
          let is_last_root_dir = matches!(ret.last(), Some(Component::RootDir));
          if is_last_root_dir {
            // If the previous path segment looks like
            // - `c:/`
            // - `/`
            // We need to ignore the `..`
          } else {
            // This branch means the previous path segment looks like
            // - `c:/a/b`
            // - `/a/b`
            ret.pop();
          }
        }
      }
      c @ Component::Normal(_) => {
        ret.push(c);
      }
    }
  }

  ret
}
