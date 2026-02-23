#[macro_export]
macro_rules! assert_eq_str {
    ($left:expr, $right:expr) => {
        assert_eq!($left.to_str().unwrap(), $right);
    };
    ($left:expr, $right:expr, $($arg:tt)*) => {
        assert_eq!($left.to_str().unwrap(), $right, $($arg)*);
    };
}

#[macro_export]
macro_rules! pb {
    ( $( $x:expr ),* ) => {
      {
        #[allow(unused_mut)]
        let mut path_buf = std::path::PathBuf::new();
        $(
          path_buf.push($x);
        )*
        path_buf
      }
    };
}

#[macro_export]
macro_rules! p {
  ( $x:expr  ) => {{
    let path = std::path::Path::new($x);
    path
  }};
}
