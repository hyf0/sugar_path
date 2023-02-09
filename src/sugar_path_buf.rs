use std::path::{Component, PathBuf};

use crate::utils::{component_vec_to_path_buf, normalize_to_component_vec};

pub trait SugarPathBuf {
    fn into_normalize(self) -> PathBuf;
}

impl SugarPathBuf for PathBuf {
    fn into_normalize(self) -> PathBuf {
        if self.as_os_str().is_empty() {
            return PathBuf::from(".");
        }

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
