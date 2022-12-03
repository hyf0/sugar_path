//! Sugar functions for manipulating paths.
//! 
//! [![document](https://docs.rs/sugar_path/badge.svg)](https://docs.rs/crate/sugar_path)
//! [![crate version](https://img.shields.io/crates/v/sugar_path.svg)](https://crates.io/crates/sugar_path) 
//! [![MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
//! 
//! 
//! - [Examples](https://github.com/iheyunfei/sugar_path/tree/main/tests)
//! - [Usages](https://docs.rs/sugar_path/latest/sugar_path/trait.SugarPath.html)

mod as_path;
pub use as_path::*;
mod sugar_path;
pub use crate::sugar_path::*;