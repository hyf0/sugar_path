use std::{
    borrow::Cow,
    path::{Component, Path, PathBuf},
};

use crate::{
    utils::{component_vec_to_path_buf, normalize_to_component_vec, CWD},
    SugarPathBuf,
};

pub trait SugarPath {
    /// normalizes the given path, resolving `'..'` and `'.'` segments.
    ///
    /// When multiple, sequential path segment separation characters are found (e.g. `/` on POSIX and either `\` or `/` on Windows), they are replaced by a single instance of the platform-specific path segment separator (`/` on POSIX and `\` on Windows). Trailing separators are preserved.
    ///
    /// If the path is a zero-length string, `'.'` is returned, representing the current working directory.
    ///
    /// If there's no normalization to be done, this function will return the original `Path`.
    ///
    /// ```rust
    /// use std::path::Path;
    /// use sugar_path::SugarPath;
    ///
    /// // For example, on POSIX:
    /// #[cfg(target_family = "unix")]
    /// assert_eq!(
    ///   Path::new("/foo/bar//baz/asdf/quux/..").normalize(),
    ///   Path::new("/foo/bar/baz/asdf")
    /// );
    ///
    /// // On Windows:
    /// #[cfg(target_family = "windows")]
    /// assert_eq!(
    ///   Path::new("C:\\temp\\\\foo\\bar\\..\\").normalize(),
    ///   Path::new("C:\\temp\\foo\\")
    /// );
    ///
    /// // Since Windows recognizes multiple path separators, both separators will be replaced by instances of the Windows preferred separator (`\`):
    /// #[cfg(target_family = "windows")]
    /// assert_eq!(
    ///   Path::new("C:////temp\\\\/\\/\\/foo/bar").normalize(),
    ///   Path::new("C:\\temp\\foo\\bar")
    /// );
    /// ```
    fn normalize(&self) -> Cow<Path>;

    /// If the path is absolute, normalize and return it.
    ///
    /// If the path is not absolute, Using CWD concat the path, normalize and return it.
    fn absolutize(&self) -> Cow<Path>;

    ///
    /// ```rust
    /// use std::path::Path;
    /// use sugar_path::SugarPath;
    /// assert_eq!(
    ///   Path::new("/var").relative("/var/lib"),
    ///   Path::new("..")
    /// );
    /// assert_eq!(
    ///   Path::new("/bin").relative("/var/lib"),
    ///   Path::new("../../bin")
    /// );
    /// assert_eq!(
    ///   Path::new("/a/b/c/d").relative("/a/b/f/g"),
    ///   Path::new("../../c/d")
    /// );
    /// ```
    fn relative(&self, to: impl AsRef<Path>) -> PathBuf;
}

impl SugarPath for Path {
    fn normalize(&self) -> Cow<Path> {
        let mut components = normalize_to_component_vec(self);
        if components.is_empty() {
            return Cow::Borrowed(Path::new("."));
        }

        if cfg!(target_family = "windows") {
            if components.len() == 1 && matches!(components[0], Component::Prefix(_)) {
                components.push(Component::CurDir)
            }
        }

        component_vec_to_path_buf(components).into()
    }

    fn absolutize(&self) -> Cow<Path> {
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
                component_vec_to_path_buf(components)
                    .into_normalize()
                    .into()
            } else {
                let mut cwd = CWD.clone();
                cwd.push(self);
                cwd.into_normalize().into()
            }
        } else {
            let mut cwd = CWD.clone();
            cwd.push(self);
            cwd.into_normalize().into()
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
                    matches!(
                        com,
                        Component::Normal(_) | Component::Prefix(_) | Component::RootDir
                    )
                })
                .collect::<Vec<_>>();
            let target_components = target
                .components()
                .into_iter()
                .filter(|com| {
                    matches!(
                        com,
                        Component::Normal(_) | Component::Prefix(_) | Component::RootDir
                    )
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
}
