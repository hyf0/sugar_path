use std::{
    fmt::format,
    path::{Component, Path, PathBuf, Prefix},
};

use once_cell::sync::Lazy;

pub(crate) static CWD: Lazy<PathBuf> = Lazy::new(|| {
    let cwd = std::env::current_dir().unwrap();
    cwd
});

pub trait PathSugar {
    /// normalizes the given path, resolving `'..'` and `'.'` segments.
    ///
    /// When multiple, sequential path segment separation characters are found (e.g. `/` on POSIX and either `\` or `/` on Windows), they are replaced by a single instance of the platform-specific path segment separator (`/` on POSIX and `\` on Windows). Trailing separators are preserved.
    ///
    /// If the path is a zero-length string, `'.'` is returned, representing the current working directory.
    ///
    /// ```rust
    /// use std::path::Path;
    /// use sugar_path::PathSugar;
    ///
    /// // For example, on POSIX:
    /// #[cfg(target_family = "unix")]
    /// assert_eq!(Path::new("/foo/bar//baz/asdf/quux/..").normalize(), Path::new("/foo/bar/baz/asdf"));
    ///
    /// // On Windows:
    /// #[cfg(target_family = "windows")]
    /// assert_eq!(Path::new("C:\\temp\\\\foo\\bar\\..\\").normalize(), Path::new("C:\\temp\\foo\\"));
    ///
    /// // Since Windows recognizes multiple path separators, both separators will be replaced by instances of the Windows preferred separator (`\`):
    /// #[cfg(target_family = "windows")]
    /// assert_eq!(Path::new("C:////temp\\\\/\\/\\/foo/bar").normalize(), Path::new("C:\\temp\\foo\\bar"));
    /// ```
    fn normalize(&self) -> PathBuf;

    /// If the path is absolute, normalize and return it.
    ///
    /// If the path is not absolute, Using CWD concat the path, normalize and return it.
    fn resolve(&self) -> PathBuf;

    fn relative(&self, base: &Path) -> PathBuf;
}

#[inline]
fn normalize_to_component_vec(path: &Path) -> Vec<Component> {
    println!("start {:?}", path);
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        vec![c]
    } else {
        vec![]
    };

    for component in components {
        println!("process {:?}", component);
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component);
            }
            Component::CurDir => {}
            c @ Component::ParentDir => {
                println!("last {:?}", ret.last());
                let is_last_none = matches!(ret.last(), None | Some(Component::Prefix(_)));
                if is_last_none {
                    ret.push(c);
                } else {
                    let is_last_root = matches!(ret.last().unwrap(), Component::RootDir);
                    println!("is_last_root {:?}", is_last_root);
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
        println!("status  {:?}", ret);
    }
    ret
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
        if cfg!(target_family = "windows") {
            // TODO: we may need to do it more delegated
            let path = PathBuf::from(self.to_string_lossy().to_string().replace("/", "\\"));
            let mut components = normalize_to_component_vec(&path);
            if components.is_empty()
                || (components.len() == 1 && matches!(components[0], Component::Prefix(_)))
            {
                components.push(Component::CurDir)
            }
            component_vec_to_path_buf(components)
        } else {
            let mut components = normalize_to_component_vec(self);
            if components.len() == 0 {
                components.push(Component::CurDir)
            }
            component_vec_to_path_buf(components)
        }
    }
    fn resolve(&self) -> PathBuf {
        if cfg!(target_family = "windows") {
            let path = PathBuf::from(self.to_string_lossy().to_string().replace("/", "\\"));
            // Consider c:
            println!("self {:?} is_absolute {:?}", self, self.is_absolute());
            println!("path {:?} is_absolute {:?}", path, path.is_absolute());
            if path.is_absolute() {
                path.normalize()
            } else {
                let mut components = path.components();
                if matches!(components.next(), Some(Component::Prefix(_)))
                    && !matches!(components.next(), Some(Component::RootDir))
                {
                    // TODO: Windows has the concept of drive-specific current working
                    // directories. If we've resolved a drive letter but not yet an
                    // absolute path, get cwd for that drive, or the process cwd if
                    // the drive cwd is not available. We're sure the device is not
                    // a UNC path at this points, because UNC paths are always absolute.
                    let mut components = path.components().into_iter().collect::<Vec<_>>();
                    components.insert(1, Component::RootDir);
                    component_vec_to_path_buf(components).normalize()
                } else {
                    let mut cwd = CWD.clone();
                    cwd.push(path);
                    cwd.normalize()
                }
            }
        } else {
            if self.is_absolute() {
                self.normalize()
            } else {
                let mut cwd = CWD.clone();
                cwd.push(self);
                cwd.normalize()
            }
        }
    }

    fn relative(&self, base: &Path) -> PathBuf {
        let from = self.resolve();
        let to = base.resolve();
        if from == to {
            PathBuf::new()
        } else {
            let from_components = from.components();
            let to_components = to.components();

            PathBuf::new()
        }
    }
}
