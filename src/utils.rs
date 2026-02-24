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
