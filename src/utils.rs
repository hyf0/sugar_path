use std::{
  borrow::Cow,
  path::{Path, PathBuf},
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
