use std::path::{Path, Component, PathBuf};


#[inline]
pub fn normalize_to_component_vec(path: &Path) -> Option<Vec<Component>> {
    let mut components = path.components().peekable();
    let mut ret = Vec::with_capacity(components.size_hint().0);
    if let Some(c @ Component::Prefix(..)) = components.peek() {
        ret.push(*c);
        components.next();
    };

    let mut has_resolved_dots = false;

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component);
            }
            Component::CurDir => {
                has_resolved_dots = true;
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
                            has_resolved_dots = true;
                            ret.pop();
                        }
                    } else {
                        has_resolved_dots = true;
                    }
                }
            }
            c @ Component::Normal(_) => {
                ret.push(c);
            }
        }
    }

    if has_resolved_dots || ret.is_empty() {
        Some(ret)
    } else {
        None
    }
}

#[inline]
pub fn component_vec_to_path_buf(components: Vec<Component>) -> PathBuf {
    components.into_iter().collect()
}
