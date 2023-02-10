use std::path::{Component, PathBuf};

use crate::utils::{component_vec_to_path_buf, normalize_to_component_vec};

pub trait SugarPathBuf {
    fn into_normalize(self) -> PathBuf;
}

impl SugarPathBuf for PathBuf {
    /// normalizes the given path, resolving `'..'` and `'.'` segments.
    ///
    /// When multiple, sequential path segment separation characters are found (e.g. `/` on POSIX and either `\` or `/` on Windows), they are replaced by a single instance of the platform-specific path segment separator (`/` on POSIX and `\` on Windows). Trailing separators are preserved.
    ///
    /// If the path is a zero-length string, `'.'` is returned, representing the current working directory.
    /// 
    /// If there's no normalization to be done, this function will return the original `PathBuf`.
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
}
