use std::{
    ffi::OsStr,
    os::unix::prelude::OsStrExt,
    path::{Component, Path, PathBuf},
};

use once_cell::sync::Lazy;

pub(crate) static CWD: Lazy<PathBuf> = Lazy::new(|| {
    let cwd = std::env::current_dir().unwrap();
    cwd
});

pub trait PathSugar {
    fn normalize(&self) -> PathBuf;
    fn resolve(&self) -> PathBuf;
}

#[inline]
fn normalize_to_component_vec(mut path: &Path) -> Vec<Component> {
    let normalize = |p: &Path| {
        let mut components = path.components().peekable();
        let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
            components.next();
            vec![c]
        } else {
            vec![]
        };

        for component in components {
            match component {
                Component::Prefix(..) => unreachable!(),
                Component::RootDir => {
                    ret.push(component);
                }
                Component::CurDir => {}
                c @ Component::ParentDir => {
                    let is_last_none = ret.last().is_none();
                    if is_last_none {
                        ret.push(c);
                    } else {
                        let is_last_root = matches!(ret.last().unwrap(), Component::RootDir);
                        if is_last_root {
                            // do nothing
                        } else {
                            let is_last_parent_dir =
                                matches!(ret.last().unwrap(), Component::ParentDir);
                            if is_last_parent_dir {
                                ret.push(c);
                            } else {
                                ret.pop();
                            }
                        }
                    }
                }
                c @ Component::Normal(_) => {
                    ret.push(c);
                }
            }
        }

        ret
    };
    if cfg!(target_family = "windows") {
        let safe = PathBuf::from(path.to_string_lossy().to_string().replace("/", "\\"));
        normalize(&safe)
    } else {
        normalize(path)
    }
}

#[inline]
fn component_vec_to_path_buf(components: Vec<Component>) -> PathBuf {
    components
        .into_iter()
        .map(|c| c.as_os_str())
        .fold(PathBuf::new(), |mut acc, cur| {
            acc.push(cur);
            acc
        })
}

impl PathSugar for Path {
    fn normalize(&self) -> PathBuf {
        let components = normalize_to_component_vec(self);
        component_vec_to_path_buf(components)
    }
    fn resolve(&self) -> PathBuf {
        let mut components = self.components().peekable();
        if components.peek().is_none() {
            CWD.clone()
        } else {
            let components = normalize_to_component_vec(self);
            if components.len() == 0 {
                CWD.clone()
            } else {
                component_vec_to_path_buf(components)
            }
        }
    }
}
