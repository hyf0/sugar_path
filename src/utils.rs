use std::path::{Component, Path, PathBuf};

use once_cell::sync::Lazy;

pub(crate) static CWD: Lazy<PathBuf> = Lazy::new(|| {
    // TODO: better way to get the current working directory?

    std::env::current_dir().unwrap()
});

#[inline]
pub fn normalize_to_component_vec(path: &Path) -> Vec<Component> {
    let mut components = path.components().peekable();
    let mut ret = Vec::with_capacity(components.size_hint().0);
    if let Some(c @ Component::Prefix(..)) = components.peek() {
        ret.push(*c);
        components.next();
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component);
            }
            Component::CurDir => {
                // ignore
            }
            c @ Component::ParentDir => {
                // For a non-absolute path `../../` or `c:../../`, we should preserve `..`
                let is_last_none_or_prefix =
                    matches!(ret.last(), None | Some(Component::Prefix(_)));
                if is_last_none_or_prefix {
                    ret.push(c);
                } else {
                    let is_last_root_dir = matches!(ret.last(), Some(Component::RootDir));
                    if !is_last_root_dir {
                        let is_last_parent_dir = matches!(ret.last(), Some(Component::ParentDir));
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
}

#[inline]
pub fn component_vec_to_path_buf(components: Vec<Component>) -> PathBuf {
    components.into_iter().collect()
}
