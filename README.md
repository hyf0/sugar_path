# sugar_path

Sugar functions for manipulating paths.

- [examples](https://github.com/iheyunfei/sugar_path/tree/main/tests)
- [docs](https://docs.rs/sugar_path/latest/sugar_path/)

# Usage

```rust
use std::path::Path;
use sugar_path::SugarPath;
assert_eq!(
  Path::new("/a/b/c/d").relative("/a/b/f/g")),
  Path::new("../../c/d")
);
```

## For Node.js developers

- basename()
  - Path#file_name()
  - Path#file_prefix()
  - Path#file_stem()
- dirname()
  - Path#parent()
- extname()
  - Path#extension()
- format()
  - wip
- isAbsolute()
  - Path#is_absolute()
- path.normalize()
  - SugarPath#normalize()
- relative()
  - SugarPath#relative()
- resolve()
  - SugarPath#resolve()

