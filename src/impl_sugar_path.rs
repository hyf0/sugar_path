use std::{
  borrow::Cow,
  ffi::OsString,
  iter::Peekable,
  ops::Deref,
  path::{Component, Path, PathBuf},
};

use memchr::{memchr, memrchr};
use smallvec::SmallVec;

use crate::{
  SugarPath,
  utils::{IntoCowPath, get_current_dir},
};

type StrVec<'a> = SmallVec<[&'a str; 8]>;

impl SugarPath for Path {
  fn normalize(&self) -> Cow<'_, Path> {
    if !needs_normalization(self) {
      return Cow::Borrowed(self);
    }
    normalize_inner(self.components().peekable(), self.as_os_str().len())
  }

  fn absolutize(&self) -> Cow<'_, Path> {
    self.absolutize_with(get_current_dir())
  }

  // Using `Cow` is on purpose.
  // - Users could choose to pass a reference or an owned value depending on their use case.
  // - If we accept `PathBuf` only, it may cause unnecessary allocations on case that `self` is already absolute.
  // - If we accept `&Path` only, it may cause unnecessary cloning that users already have an owned value.
  //
  // NOTE: we intentionally keep the return lifetime tied to `&self` (not `'a`).
  // Unifying them (`&'a self, impl IntoCowPath<'a>) -> Cow<'a, ...>`) would allow
  // borrowing from `base` for noop cases ("", "."), but it constrains callers:
  // base's borrowed data must outlive self. That's a semver-breaking trade-off
  // for a narrow benefit — callers needing "".absolutize_with(base) can just
  // call base.normalize() directly.
  fn absolutize_with<'a>(&self, base: impl IntoCowPath<'a>) -> Cow<'_, Path> {
    if self.is_absolute() {
      return self.normalize();
    }

    let base: Cow<'a, Path> = base.into_cow_path();
    let mut base =
      if base.is_absolute() { base } else { Cow::Owned(base.absolutize().into_owned()) };

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
        let mut components: SmallVec<[Component; 8]> = components.collect();
        components.insert(1, Component::RootDir);
        Cow::Owned(
          normalize_inner(components.into_iter().peekable(), self.as_os_str().len()).into_owned(),
        )
      } else {
        base.to_mut().push(self);
        Cow::Owned(base.normalize().into_owned())
      }
    } else {
      base.to_mut().push(self);
      Cow::Owned(base.normalize().into_owned())
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
          return self.normalize().into_owned();
        }
        // Unrecognized prefix format: fall through to component-based path
      }
      #[cfg(not(target_family = "windows"))]
      {
        return PathBuf::from(relative_str(target_str, base_str));
      }
    }

    // Slow path: avoid current_dir() syscall for already-absolute paths
    let base = if base_ref.is_absolute() {
      base_ref.normalize().into_owned()
    } else {
      base_ref.absolutize().into_owned()
    };
    let target = if self.is_absolute() {
      self.normalize().into_owned()
    } else {
      self.absolutize().into_owned()
    };
    if base == target {
      PathBuf::new()
    } else {
      // Filter components inline
      let filter_fn = |com: &Component| {
        matches!(com, Component::Normal(_) | Component::Prefix(_) | Component::RootDir)
      };

      // Iterate components without intermediate allocation
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

/// Check whether a path needs normalization. Returns `false` for already-clean
/// paths, allowing `normalize()` to return `Cow::Borrowed` with zero allocation.
#[inline]
#[cfg(not(target_family = "windows"))]
fn needs_normalization(path: &Path) -> bool {
  let Some(s) = path.to_str() else {
    return true;
  };
  let bytes = s.as_bytes();
  if bytes.is_empty() {
    return true;
  }
  // Leading `.` or `..` component (but not `...` or `.foo` which are normal filenames)
  if bytes[0] == b'.' {
    if bytes.len() == 1 || bytes[1] == b'/' {
      return true;
    }
    if bytes[1] == b'.' && (bytes.len() == 2 || bytes[2] == b'/') {
      return true;
    }
  }
  // Trailing `/` (unless path is exactly `/`)
  if bytes.len() > 1 && bytes[bytes.len() - 1] == b'/' {
    return true;
  }
  // memchr scan for `//`, `/.`, `/..`
  let mut offset = 0;
  while let Some(pos) = memchr(b'/', &bytes[offset..]) {
    let slash = offset + pos;
    let next = slash + 1;
    if next < bytes.len() {
      let b = bytes[next];
      // `//` — consecutive slashes
      if b == b'/' {
        return true;
      }
      // `/.` — could be `/.` or `/..`
      if b == b'.' {
        let after_dot = next + 1;
        // "/." at end or "/./"
        if after_dot >= bytes.len() || bytes[after_dot] == b'/' {
          return true;
        }
        // "/.." at end or "/../"
        if bytes[after_dot] == b'.'
          && (after_dot + 1 >= bytes.len() || bytes[after_dot + 1] == b'/')
        {
          return true;
        }
      }
    }
    offset = next;
  }
  false
}

/// Check whether a path needs normalization (Windows variant).
#[inline]
#[cfg(target_family = "windows")]
fn needs_normalization(path: &Path) -> bool {
  let Some(s) = path.to_str() else {
    return true;
  };
  let bytes = s.as_bytes();
  if bytes.is_empty() {
    return true;
  }
  // Any forward slash means normalization is needed (gets converted to `\`)
  if memchr(b'/', bytes).is_some() {
    return true;
  }
  // UNC prefix `\\` at start — always bail out to normalizer
  if bytes.len() >= 2 && bytes[0] == b'\\' && bytes[1] == b'\\' {
    return true;
  }
  // Bare drive `X:` without trailing `\` normalizes to `X:.`
  // Also `X:foo` (drive-relative) needs normalization
  if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
    if bytes.len() == 2 {
      return true; // bare `C:`
    }
    if bytes[2] != b'\\' {
      return true; // `C:foo` (drive-relative, no root)
    }
  }
  // Leading `.` or `..` component (but not `...` or `.foo` which are normal filenames)
  if bytes[0] == b'.' {
    if bytes.len() == 1 || bytes[1] == b'\\' {
      return true;
    }
    if bytes[1] == b'.' && (bytes.len() == 2 || bytes[2] == b'\\') {
      return true;
    }
  }
  // Trailing `\` (unless path is `\` alone or `X:\`)
  if bytes[bytes.len() - 1] == b'\\' {
    // `\` alone is clean
    if bytes.len() == 1 {
      return false;
    }
    // `X:\` is clean
    if bytes.len() == 3 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
      return false;
    }
    return true;
  }
  // memchr scan for `\\` (consecutive), `\.`, `\..`
  let mut offset = 0;
  while let Some(pos) = memchr(b'\\', &bytes[offset..]) {
    let slash = offset + pos;
    let next = slash + 1;
    if next < bytes.len() {
      let b = bytes[next];
      // `\\` — consecutive separators
      if b == b'\\' {
        return true;
      }
      // `\.` — could be `\.` or `\..`
      if b == b'.' {
        let after_dot = next + 1;
        // "\." at end or "\.\"
        if after_dot >= bytes.len() || bytes[after_dot] == b'\\' {
          return true;
        }
        // "\.." at end or "\..\"
        if bytes[after_dot] == b'.'
          && (after_dot + 1 >= bytes.len() || bytes[after_dot + 1] == b'\\')
        {
          return true;
        }
      }
    }
    offset = next;
  }
  false
}

#[inline]
fn normalize_inner<'a>(
  mut components: Peekable<impl Iterator<Item = Component<'a>>>,
  hint_cap: usize,
) -> Cow<'a, Path> {
  let sep_byte = std::path::MAIN_SEPARATOR as u8;
  let mut buf: Vec<u8> = Vec::with_capacity(hint_cap);
  let mut has_root = false;
  let mut depth: usize = 0; // count of Normal segments currently in buf
  let mut need_sep = false;

  // --- Prefix (Windows only) ---
  #[cfg(target_family = "windows")]
  let prefix_len: usize;
  #[cfg(target_family = "windows")]
  {
    if let Some(Component::Prefix(p)) = components.peek() {
      if let std::path::Prefix::UNC(server, share) = p.kind() {
        buf.extend_from_slice(b"\\\\");
        buf.extend_from_slice(server.as_encoded_bytes());
        buf.push(b'\\');
        buf.extend_from_slice(share.as_encoded_bytes());
      } else {
        buf.extend_from_slice(p.as_os_str().as_encoded_bytes());
      }
      components.next();
    }
    prefix_len = buf.len();
  }

  // --- RootDir ---
  if matches!(components.peek(), Some(Component::RootDir)) {
    buf.push(sep_byte);
    has_root = true;
    components.next();
  }

  let root_end = buf.len();

  // --- Remaining components ---
  for component in components {
    match component {
      Component::Prefix(prefix) => unreachable!("Unexpected prefix for {:?}", prefix),
      Component::RootDir => unreachable!("Unexpected RootDir after initial position"),
      Component::CurDir => {}
      Component::ParentDir => {
        if depth > 0 {
          // Roll back the last Normal segment using memrchr.
          let search_region = &buf[root_end..];
          if let Some(pos) = memrchr(sep_byte, search_region) {
            buf.truncate(root_end + pos);
          } else {
            buf.truncate(root_end);
          }
          depth -= 1;
          need_sep = buf.len() > root_end;
        } else if !has_root {
          // Relative path going above start: write ".." literally
          if need_sep {
            buf.push(sep_byte);
          }
          buf.extend_from_slice(b"..");
          need_sep = true;
        }
        // else: has_root && depth == 0 → ignore (can't go above root)
      }
      Component::Normal(s) => {
        if need_sep {
          buf.push(sep_byte);
        }
        buf.extend_from_slice(s.as_encoded_bytes());
        depth += 1;
        need_sep = true;
      }
    }
  }

  // --- Empty result → "." ---
  if buf.is_empty() {
    return Cow::Borrowed(Path::new("."));
  }

  // --- Prefix-only: append trailing separator or CurDir ---
  #[cfg(target_family = "windows")]
  if buf.len() == prefix_len && prefix_len > 0 {
    // Determine if the prefix is UNC by checking for leading "\\"
    if buf.len() >= 2 && buf[0] == b'\\' && buf[1] == b'\\' {
      buf.push(b'\\');
    } else {
      buf.push(b'.');
    }
  }

  // SAFETY: `buf` was built entirely from:
  // - encoded bytes of OsStr components (valid platform encoding)
  // - ASCII separator bytes and ASCII '.' characters
  // This preserves the encoding invariants required by OsString.
  Cow::Owned(PathBuf::from(unsafe { OsString::from_encoded_bytes_unchecked(buf) }))
}

impl<T: Deref<Target = str>> SugarPath for T {
  fn normalize(&self) -> Cow<'_, Path> {
    self.as_path().normalize()
  }

  fn absolutize(&self) -> Cow<'_, Path> {
    self.as_path().absolutize()
  }

  fn absolutize_with<'a>(&self, base: impl IntoCowPath<'a>) -> Cow<'_, Path> {
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
