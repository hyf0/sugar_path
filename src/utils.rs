use std::{borrow::Cow, io, path::Path};

#[cfg(feature = "cached_current_dir")]
use std::{path::PathBuf, sync::OnceLock};

#[cfg(feature = "cached_current_dir")]
static CURRENT_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn try_get_current_dir() -> io::Result<Cow<'static, Path>> {
  #[cfg(feature = "cached_current_dir")]
  {
    if let Some(current_dir) = CURRENT_DIR.get() {
      return Ok(Cow::Borrowed(current_dir));
    }

    let current_dir = std::env::current_dir()?;
    let _ = CURRENT_DIR.set(current_dir);
    Ok(Cow::Borrowed(
      CURRENT_DIR.get().expect("the current-directory cache was initialized by this call"),
    ))
  }

  #[cfg(not(feature = "cached_current_dir"))]
  {
    std::env::current_dir().map(Cow::Owned)
  }
}
