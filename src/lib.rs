use std::path::{Component, Path, PathBuf};

use once_cell::sync::Lazy;

pub(crate) static POSIX_CWD: Lazy<PathBuf> = Lazy::new(|| {
    let mut cwd = std::env::current_dir().unwrap();
    cwd
});

pub trait PathSugar {
    fn normalize(&self) -> PathBuf;
}

impl PathSugar for Path {
    fn normalize(&self) -> PathBuf {
        let mut components = self.components().peekable();
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
        let p = ret
            .into_iter()
            .map(|c| c.as_os_str())
            .fold(PathBuf::new(), |mut acc, cur| {
                acc.push(cur);
                acc
            });
        p
    }
}
