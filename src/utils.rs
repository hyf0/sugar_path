use std::path::{Component, Path, PathBuf};


#[inline]
pub fn component_vec_to_path_buf(components: Vec<Component>) -> PathBuf {
  components.into_iter().collect()
}

pub fn to_normalized_components(path: &Path) -> Vec<Component> {
  let mut components = path.components().peekable();
  let mut ret = Vec::with_capacity(components.size_hint().0);
  if let Some(c @ Component::Prefix(..)) = components.peek() {
    ret.push(*c);
    components.next();
  };

  for component in components {
    match component {
      Component::Prefix(..) => unreachable!("Unexpected prefix for {:?}", path.display()),
      Component::RootDir => {
        ret.push(component);
      }
      Component::CurDir => {
        // ignore
      }
      c @ Component::ParentDir => {
        // So we hit a `..` here. If the previous path segment looks like
        // - `c:`
        // - `c:../..`
        // - `../..`
        // - ``
        // We should preserve the `..`

        let need_to_preserve =
          matches!(ret.last(), None | Some(Component::Prefix(_)) | Some(Component::ParentDir));
        if need_to_preserve {
          ret.push(c);
        } else {
          let is_last_root_dir = matches!(ret.last(), Some(Component::RootDir));
          if is_last_root_dir {
            // If the previous path segment looks like
            // - `c:/`
            // - `/`
            // We need to ignore the `..`
          } else {
            // This branch means the previous path segment looks like
            // - `c:/a/b`
            // - `/a/b`
            ret.pop();
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
