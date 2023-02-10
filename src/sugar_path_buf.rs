use std::path::{Component, PathBuf};

use crate::utils::{component_vec_to_path_buf, normalize_to_component_vec, CWD};

pub trait SugarPathBuf {
    /// normalizes the given path, resolving `'..'` and `'.'` segments.
    ///
    /// When multiple, sequential path segment separation characters are found (e.g. `/` on POSIX and either `\` or `/` on Windows), they are replaced by a single instance of the platform-specific path segment separator (`/` on POSIX and `\` on Windows). Trailing separators are preserved.
    ///
    /// If the path is a zero-length string, `'.'` is returned, representing the current working directory.
    ///
    /// If there's no normalization to be done, this function will return the original `PathBuf`.
    fn into_normalize(self) -> PathBuf;

    fn into_absolutize(self) -> PathBuf;
}

impl SugarPathBuf for PathBuf {
    fn into_normalize(self) -> PathBuf {
        let components = normalize_to_component_vec(&self);

        if let Some(mut components) = components {
            if components.is_empty() {
                return PathBuf::from(".");
            }

            if cfg!(target_family = "windows") {
                if components.len() == 1 && matches!(components[0], Component::Prefix(_)) {
                    components.push(Component::CurDir)
                }
            }

            component_vec_to_path_buf(components)
        } else {
            self
        }
    }

    /// If there's no absolutization to be done, this function will return the original `PathBuf`.
    fn into_absolutize(self) -> PathBuf {
        if self.is_absolute() {
            self.into_normalize()
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
                component_vec_to_path_buf(components).into_normalize()
            } else {
                let mut cwd = CWD.clone();
                cwd.push(self);
                cwd.into_normalize()
            }
        } else {
            let mut cwd = CWD.clone();
            cwd.push(self);
            cwd.into_normalize()
        }
    }
}
