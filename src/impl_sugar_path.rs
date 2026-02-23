use std::{
  borrow::Cow,
  ops::Deref,
  path::{Component, Path, PathBuf},
};

use memchr::{memchr, memrchr};
use smallvec::SmallVec;

use crate::{
  SugarPath,
  utils::{ComponentVec, IntoCowPath, get_current_dir, to_normalized_components},
};

type StrVec<'a> = SmallVec<[&'a str; 8]>;

impl SugarPath for Path {
  fn normalize(&self) -> PathBuf {
    let peekable = self.components().peekable();
    let mut components = to_normalized_components(peekable);

    normalize_inner(&mut components)
  }

  fn absolutize(&self) -> PathBuf {
    self.absolutize_with(get_current_dir())
  }

  // Using `Cow` is on purpose.
  // - Users could choose to pass a reference or an owned value depending on their use case.
  // - If we accept `PathBuf` only, it may cause unnecessary allocations on case that `self` is already absolute.
  // - If we accept `&Path` only, it may cause unnecessary cloning that users already have an owned value.
  fn absolutize_with<'a>(&self, base: impl IntoCowPath<'a>) -> PathBuf {
    if self.is_absolute() {
      return self.normalize();
    }

    let base: Cow<'a, Path> = base.into_cow_path();
    let mut base = if base.is_absolute() { base } else { Cow::Owned(base.absolutize()) };

    if cfg!(target_family = "windows") {
      // Consider c:
      let mut components = self.components().peekable();
      if matches!(components.peek(), Some(Component::Prefix(_)))
        && !matches!(components.peek(), Some(Component::RootDir))
      {
        // TODO: Windows has the concept of drive-specific current working
        // directories. If we've resolved a drive letter but not yet an
        // absolute path, get cwd for that drive, or the process cwd if
        // the drive cwd is not available. We're sure the device is not
        // a UNC path at this points, because UNC paths are always absolute.
        let mut components: ComponentVec = components.collect();
        components.insert(1, Component::RootDir);
        let mut components = to_normalized_components(components.into_iter().peekable());
        normalize_inner(&mut components)
      } else {
        base.to_mut().push(self);
        base.normalize()
      }
    } else {
      base.to_mut().push(self);
      base.normalize()
    }
  }

  fn relative(&self, to: impl AsRef<Path>) -> PathBuf {
    let base_ref = to.as_ref();

    // Fast path: when both paths are absolute and valid UTF-8, use
    // memchr-accelerated string operations to avoid absolutize() overhead
    // and intermediate PathBuf allocations.
    if self.is_absolute()
      && base_ref.is_absolute()
      && let (Some(target_str), Some(base_str)) = (self.to_str(), base_ref.to_str())
    {
      #[cfg(target_family = "windows")]
      {
        let target_fwd = normalize_backslash_cow(target_str);
        let base_fwd = normalize_backslash_cow(base_str);
        if let (Some((target_root, target_rest)), Some((base_root, base_rest))) =
          (split_windows_root(&target_fwd), split_windows_root(&base_fwd))
        {
          if target_root.eq_ignore_ascii_case(base_root) {
            // Same root: compute relative path with case-insensitive comparison
            let result = relative_str(target_rest, base_rest);
            return PathBuf::from(result.replace('/', "\\"));
          }
          // Different roots: return normalized target (no current_dir needed)
          return self.normalize();
        }
        // Unrecognized prefix format: fall through to component-based path
      }
      #[cfg(not(target_family = "windows"))]
      {
        return PathBuf::from(relative_str(target_str, base_str));
      }
    }

    // Slow path: avoid current_dir() syscall for already-absolute paths
    let base = if base_ref.is_absolute() { base_ref.normalize() } else { base_ref.absolutize() };
    let target = if self.is_absolute() { self.normalize() } else { self.absolutize() };
    if base == target {
      PathBuf::new()
    } else {
      // Filter components inline
      let filter_fn = |com: &Component| {
        matches!(com, Component::Normal(_) | Component::Prefix(_) | Component::RootDir)
      };

      // Collect components using SmallVec to avoid heap allocation for typical paths
      let base_components = base.components().filter(filter_fn);
      let target_components = target.components().filter(filter_fn);

      // Find common prefix length
      let common_len = base_components
        .clone()
        .zip(target_components.clone())
        .take_while(|(from, to)| {
          // Handle Windows case-insensitive comparison
          if cfg!(target_family = "windows")
            && let (Component::Normal(from_seg), Component::Normal(to_seg)) = (from, to)
          {
            return from_seg.eq_ignore_ascii_case(to_seg);
          }
          from == to
        })
        .count();

      // Build the result path without repeated PathBuf::push allocations
      let up_len = base_components.count().saturating_sub(common_len);

      (0..up_len).map(|_| Component::ParentDir).chain(target_components.skip(common_len)).collect()
    }
  }

  fn to_slash<'a>(&'a self) -> Option<Cow<'a, str>> {
    if std::path::MAIN_SEPARATOR == '/' {
      self.to_str().map(Cow::Borrowed)
    } else {
      self.to_str().map(|s| match replace_main_separator(s) {
        Some(replaced) => Cow::Owned(replaced),
        None => Cow::Borrowed(s),
      })
    }
  }

  fn to_slash_lossy<'a>(&'a self) -> Cow<'a, str> {
    if std::path::MAIN_SEPARATOR == '/' {
      self.to_string_lossy()
    } else {
      match self.to_string_lossy() {
        Cow::Borrowed(s) => match replace_main_separator(s) {
          Some(replaced) => Cow::Owned(replaced),
          None => Cow::Borrowed(s),
        },
        Cow::Owned(owned) => match replace_main_separator(&owned) {
          Some(replaced) => Cow::Owned(replaced),
          None => Cow::Owned(owned),
        },
      }
    }
  }

  fn as_path(&self) -> &Path {
    self
  }
}

#[inline]
fn normalize_inner(components: &mut ComponentVec) -> PathBuf {
  if components.is_empty() {
    return PathBuf::from(".");
  }

  if cfg!(target_family = "windows")
    && components.len() == 1
    && matches!(components[0], Component::Prefix(_))
  {
    components.push(Component::CurDir)
  }

  components.iter().collect()
}

impl<T: Deref<Target = str>> SugarPath for T {
  fn normalize(&self) -> PathBuf {
    self.as_path().normalize()
  }

  fn absolutize(&self) -> PathBuf {
    self.as_path().absolutize()
  }

  fn absolutize_with<'a>(&self, base: impl IntoCowPath<'a>) -> PathBuf {
    self.as_path().absolutize_with(base)
  }

  fn relative(&self, to: impl AsRef<Path>) -> PathBuf {
    self.as_path().relative(to)
  }

  fn to_slash<'a>(&'a self) -> Option<Cow<'a, str>> {
    self.as_path().to_slash()
  }

  fn to_slash_lossy<'a>(&'a self) -> Cow<'a, str> {
    self.as_path().to_slash_lossy()
  }

  fn as_path(&self) -> &Path {
    Path::new(self.deref())
  }
}

/// String-based relative path computation. Dispatches to the fast path when
/// no `.`/`..` normalization is needed, otherwise normalizes first.
fn relative_str(target: &str, base: &str) -> String {
  if needs_dot_normalization(target) || needs_dot_normalization(base) {
    relative_str_slow(target, base)
  } else {
    relative_str_fast(target, base)
  }
}

/// Check if a path contains `.` or `..` components that need normalization.
/// Uses `memchr` to jump between `/` positions — most bytes in a path aren't `/`,
/// so this skips the vast majority of the input.
#[inline]
fn needs_dot_normalization(path: &str) -> bool {
  let bytes = path.as_bytes();
  let mut offset = 0;
  while let Some(pos) = memchr(b'/', &bytes[offset..]) {
    let slash = offset + pos;
    if slash + 1 < bytes.len() && bytes[slash + 1] == b'.' {
      let after_dot = slash + 2;
      // "/." at end or "/./"
      if after_dot >= bytes.len() || bytes[after_dot] == b'/' {
        return true;
      }
      // "/.." at end or "/../"
      if bytes[after_dot] == b'.' && (after_dot + 1 >= bytes.len() || bytes[after_dot + 1] == b'/')
      {
        return true;
      }
    }
    offset = slash + 1;
  }
  false
}

/// Fast path: no normalization needed. Operates directly on `&str` slices
/// with zero intermediate allocation.
fn relative_str_fast(target: &str, base: &str) -> String {
  let common_byte_len = {
    #[cfg(target_family = "windows")]
    {
      target
        .as_bytes()
        .iter()
        .zip(base.as_bytes().iter())
        .take_while(|(a, b)| a.eq_ignore_ascii_case(b))
        .count()
    }
    #[cfg(not(target_family = "windows"))]
    {
      target.bytes().zip(base.bytes()).take_while(|(a, b)| a == b).count()
    }
  };

  // Adjust to last '/' boundary to ensure we match full path components
  // Check if common_byte_len falls on a component boundary:
  // - exact match (both exhausted)
  // - one side exhausted and the other has '/' next (prefix match)
  let at_boundary = (common_byte_len == target.len() && common_byte_len == base.len())
    || (common_byte_len == target.len() && base.as_bytes().get(common_byte_len) == Some(&b'/'))
    || (common_byte_len == base.len() && target.as_bytes().get(common_byte_len) == Some(&b'/'));
  let common_prefix = if at_boundary {
    common_byte_len
  } else {
    memrchr(b'/', &target.as_bytes()[..common_byte_len]).unwrap_or(0)
  };

  // Count remaining base components
  let base_remaining = &base.as_bytes()[common_prefix..];
  let mut ups = 0u32;
  {
    let mut offset = 0;
    while offset < base_remaining.len() {
      if base_remaining[offset] == b'/' {
        offset += 1;
        continue;
      }
      ups += 1;
      offset = match memchr(b'/', &base_remaining[offset..]) {
        Some(pos) => offset + pos + 1,
        None => base_remaining.len(),
      };
    }
  }

  let target_suffix = target[common_prefix..].trim_start_matches('/');
  let ups = ups as usize;
  let suffix_iter = if target_suffix.is_empty() { None } else { Some(target_suffix) };
  let mut result = String::with_capacity(ups * 3 + target_suffix.len());
  std::iter::repeat_n("..", ups).chain(suffix_iter).for_each(|s| {
    if !result.is_empty() {
      result.push('/');
    }
    result.push_str(s);
  });
  result
}

/// Slow path: normalize `.` and `..` components first, then compute relative path.
fn relative_str_slow(target: &str, base: &str) -> String {
  let target_parts = normalize_parts(target);
  let base_parts = normalize_parts(base);

  let common_len = {
    #[cfg(target_family = "windows")]
    {
      target_parts
        .iter()
        .zip(base_parts.iter())
        .take_while(|(a, b)| a.eq_ignore_ascii_case(b))
        .count()
    }
    #[cfg(not(target_family = "windows"))]
    {
      target_parts.iter().zip(base_parts.iter()).take_while(|(a, b)| a == b).count()
    }
  };

  let ups = base_parts.len() - common_len;
  let remaining = &target_parts[common_len..];

  let remaining_len: usize =
    remaining.iter().map(|s| s.len()).sum::<usize>() + remaining.len().saturating_sub(1);
  let mut result = String::with_capacity(ups * 3 + remaining_len);
  std::iter::repeat_n("..", ups).chain(remaining.iter().copied()).for_each(|s| {
    if !result.is_empty() {
      result.push('/');
    }
    result.push_str(s);
  });
  result
}

/// Split a path into normalized components, resolving `.` and `..` lexically.
fn normalize_parts(path: &str) -> StrVec<'_> {
  let mut parts = StrVec::new();
  for part in path.split('/') {
    match part {
      "" | "." => {}
      ".." => {
        parts.pop();
      }
      _ => parts.push(part),
    }
  }
  parts
}

/// Replace `\` with `/` using memchr SIMD search. Returns the input unchanged
/// (zero allocation) when no backslashes are present.
#[cfg(target_family = "windows")]
fn normalize_backslash_cow(s: &str) -> Cow<'_, str> {
  let bytes = s.as_bytes();
  let Some(first) = memchr(b'\\', bytes) else {
    return Cow::Borrowed(s);
  };
  let mut out = Vec::with_capacity(bytes.len());
  out.extend_from_slice(&bytes[..first]);
  out.push(b'/');
  let mut offset = first + 1;
  while let Some(pos) = memchr(b'\\', &bytes[offset..]) {
    out.extend_from_slice(&bytes[offset..offset + pos]);
    out.push(b'/');
    offset += pos + 1;
  }
  out.extend_from_slice(&bytes[offset..]);
  // SAFETY: input is valid UTF-8, and we only replaced `\` (single ASCII byte) with `/`
  Cow::Owned(unsafe { String::from_utf8_unchecked(out) })
}

/// Extract the Windows root prefix and remaining path from a forward-slash-normalized path.
/// Returns `(root, rest)` where root excludes the trailing separator.
///
/// - Drive: `"c:/foo/bar"` → `("c:", "foo/bar")`
/// - UNC:   `"//server/share/foo"` → `("//server/share", "foo")`
#[cfg(target_family = "windows")]
fn split_windows_root(path: &str) -> Option<(&str, &str)> {
  let bytes = path.as_bytes();
  if bytes.len() >= 2 && bytes[1] == b':' {
    // Drive letter: c:/...
    let rest_start = if bytes.get(2) == Some(&b'/') { 3 } else { 2 };
    Some((&path[..2], &path[rest_start..]))
  } else if bytes.len() >= 2 && bytes[0] == b'/' && bytes[1] == b'/' {
    // UNC: //server/share/...
    let server_end = memchr(b'/', &bytes[2..]).map(|p| 2 + p)?;
    let share_start = server_end + 1;
    let share_end =
      memchr(b'/', &bytes[share_start..]).map(|p| share_start + p).unwrap_or(bytes.len());
    let rest_start = if share_end < bytes.len() { share_end + 1 } else { share_end };
    Some((&path[..share_end], &path[rest_start..]))
  } else {
    None
  }
}

fn replace_main_separator(input: &str) -> Option<String> {
  let sep = std::path::MAIN_SEPARATOR;
  let mut replaced: Option<String> = None;
  let mut segment_start = 0;

  for (idx, ch) in input.char_indices() {
    if ch == sep {
      let buf = replaced.get_or_insert_with(|| String::with_capacity(input.len()));
      buf.push_str(&input[segment_start..idx]);
      buf.push('/');
      segment_start = idx + ch.len_utf8();
    }
  }

  if let Some(mut buf) = replaced {
    buf.push_str(&input[segment_start..]);
    Some(buf)
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use std::{borrow::Cow, path::Path, path::PathBuf};

  use super::SugarPath;

  #[allow(unused_macros)]
  macro_rules! assert_eq_str {
    ($left:expr, $right:expr) => {
      assert_eq!($left.to_str().unwrap(), $right);
    };
    ($left:expr, $right:expr, $($arg:tt)*) => {
      assert_eq!($left.to_str().unwrap(), $right, $($arg)*);
    };
  }

  #[test]
  fn _test_as_path() {
    let str = "";
    str.as_path();

    let string = String::new();
    string.as_path();

    let ref_string = &string;
    ref_string.as_path();
  }

  #[test]
  fn _test_absolutize_with() {
    let tmp = "";

    let str = "";
    tmp.absolutize_with(str);

    let string = String::new();
    tmp.absolutize_with(string);

    let ref_string = &String::new();
    tmp.absolutize_with(ref_string);

    let path = Path::new("");
    tmp.absolutize_with(path);

    let path_buf = PathBuf::new();
    tmp.absolutize_with(path_buf);

    let cow_path = Cow::Borrowed(Path::new(""));
    tmp.absolutize_with(cow_path);

    let cow_str = Cow::Borrowed("");
    tmp.absolutize_with(cow_str);
  }

  #[cfg(target_family = "unix")]
  #[test]
  fn normalize() {
    assert_eq_str!(Path::new("/foo/../../../bar").normalize(), "/bar");
    assert_eq_str!(Path::new("a//b//../b").normalize(), "a/b");
    assert_eq_str!(Path::new("/foo/../../../bar").normalize(), "/bar");
    assert_eq_str!(Path::new("a//b//./c").normalize(), "a/b/c");
    assert_eq_str!(Path::new("a//b//.").normalize(), "a/b");
    assert_eq_str!(Path::new("/a/b/c/../../../x/y/z").normalize(), "/x/y/z");
    assert_eq_str!(Path::new("///..//./foo/.//bar").normalize(), "/foo/bar");
    assert_eq_str!(Path::new("bar/foo../../").normalize(), "bar");
    assert_eq_str!(Path::new("bar/foo../..").normalize(), "bar");
    assert_eq_str!(Path::new("bar/foo../../baz").normalize(), "bar/baz");
    assert_eq_str!(Path::new("bar/foo../").normalize(), "bar/foo..");
    assert_eq_str!(Path::new("bar/foo..").normalize(), "bar/foo..");
    assert_eq_str!(Path::new("../foo../../../bar").normalize(), "../../bar");
    assert_eq_str!(Path::new("../foo../../../bar").normalize(), "../../bar");
    assert_eq_str!(Path::new("../.../.././.../../../bar").normalize(), "../../bar");
    assert_eq_str!(Path::new("../.../.././.../../../bar").normalize(), "../../bar");
    assert_eq_str!(Path::new("../../../foo/../../../bar").normalize(), "../../../../../bar");
    assert_eq_str!(Path::new("../../../foo/../../../bar/../../").normalize(), "../../../../../..");
    assert_eq_str!(Path::new("../foobar/barfoo/foo/../../../bar/../../").normalize(), "../..");
    assert_eq_str!(
      Path::new("../.../../foobar/../../../bar/../../baz").normalize(),
      "../../../../baz"
    );
    assert_eq_str!(Path::new("foo/bar\\baz").normalize(), "foo/bar\\baz");
    assert_eq_str!(Path::new("/a/b/c/../../../").normalize(), "/");
    assert_eq_str!(Path::new("a/b/c/../../../").normalize(), ".");
    assert_eq_str!(Path::new("a/b/c/../../..").normalize(), ".");

    assert_eq_str!(Path::new("").normalize(), ".");
  }
}
