use std::{
  borrow::Cow,
  ffi::{OsStr, OsString},
  io,
  iter::Peekable,
  path::{Component, Path, PathBuf},
};

use memchr::{memchr, memrchr};
use smallvec::SmallVec;

use crate::{SugarPath, utils::try_get_current_dir};

type StrVec<'a> = SmallVec<[&'a str; 8]>;
type OsStrVec<'a> = SmallVec<[&'a OsStr; 16]>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TrailingSeparator {
  Preserve,
  Strip,
}

/// Stack arena for owned normalize component bytes. Paths longer than this fall
/// back to `normalize_inner` (one heap allocation — same floor as before).
const OWNED_NORMALIZE_STACK_ARENA: usize = 512;

enum RelativeOutcome<'a> {
  BorrowedNative(&'a Path),
  Native(PathBuf),
  Slash(String),
}

impl<'a> RelativeOutcome<'a> {
  fn into_path_buf(self) -> PathBuf {
    match self {
      Self::BorrowedNative(path) => path.to_owned(),
      Self::Native(path) => path,
      Self::Slash(path) => {
        #[cfg(target_family = "windows")]
        {
          PathBuf::from(replace_forward_separator_in_owned(path))
        }
        #[cfg(not(target_family = "windows"))]
        {
          PathBuf::from(path)
        }
      }
    }
  }

  fn into_cow_path(self) -> Cow<'a, Path> {
    match self {
      Self::BorrowedNative(path) => Cow::Borrowed(path),
      outcome => Cow::Owned(outcome.into_path_buf()),
    }
  }
}

#[cfg(any(test, target_family = "windows"))]
fn replace_forward_separator_in_owned(string: String) -> String {
  let mut bytes = string.into_bytes();
  for byte in &mut bytes {
    if *byte == b'/' {
      *byte = b'\\';
    }
  }
  String::from_utf8(bytes).expect("replacing ASCII path separators preserves UTF-8")
}

#[cfg(test)]
#[test]
fn owned_forward_separator_replacement_is_exact_and_reuses_storage() {
  assert_eq!(replace_forward_separator_in_owned(String::new()), "");

  for expected in ["mod.rs", r"..\src\mod.rs", r"模块\src\任务.rs"] {
    let input = expected.replace('\\', "/");
    let allocation = (input.as_ptr(), input.capacity());
    let output = replace_forward_separator_in_owned(input);
    assert_eq!(output, expected);
    assert_eq!((output.as_ptr(), output.capacity()), allocation);
  }
}

#[derive(Clone, Copy)]
struct LexicalRelativeShape {
  unresolved_parents: usize,
  surviving_normals: usize,
  max_normal_depth: usize,
}

fn classify_lexical_relative(path: &Path) -> Option<LexicalRelativeShape> {
  let mut unresolved_parents = 0;
  let mut surviving_normals = 0;
  let mut max_normal_depth = 0;

  for component in path.components() {
    match component {
      Component::CurDir => {}
      Component::ParentDir => {
        if surviving_normals > 0 {
          surviving_normals -= 1;
        } else {
          unresolved_parents += 1;
        }
      }
      Component::Normal(_) => {
        surviving_normals += 1;
        max_normal_depth = max_normal_depth.max(surviving_normals);
      }
      Component::Prefix(_) | Component::RootDir => return None,
    }
  }

  Some(LexicalRelativeShape { unresolved_parents, surviving_normals, max_normal_depth })
}

fn collect_lexical_normals<'a>(path: &'a Path, shape: LexicalRelativeShape) -> OsStrVec<'a> {
  let mut normals = OsStrVec::with_capacity(shape.max_normal_depth);
  for component in path.components() {
    match component {
      Component::CurDir => {}
      Component::ParentDir => {
        normals.pop();
      }
      Component::Normal(normal) => normals.push(normal),
      Component::Prefix(_) | Component::RootDir => {
        unreachable!("classified lexical relative paths have no prefix or root")
      }
    }
  }
  debug_assert_eq!(normals.len(), shape.surviving_normals);
  normals
}

fn try_relative_lexically(target: &Path, base: &Path) -> Option<PathBuf> {
  let target_shape = classify_lexical_relative(target)?;
  let base_shape = classify_lexical_relative(base)?;
  if target_shape.unresolved_parents != base_shape.unresolved_parents {
    return None;
  }

  let target = collect_lexical_normals(target, target_shape);
  let base = collect_lexical_normals(base, base_shape);
  relative_from_normal_stacks(&base, &target)
}

/// Walk an absolute path into its normal-component stack under the root (and
/// Windows prefix). CurDir is ignored; ParentDir pops; RootDir clears normals.
/// Pure relative inputs share this stack when resolved against one cwd.
fn absolute_normal_stack<'a>(path: &'a Path) -> Option<OsStrVec<'a>> {
  if !path.is_absolute() {
    return None;
  }

  let mut stack = OsStrVec::with_capacity(16);
  for component in path.components() {
    match component {
      Component::Prefix(_) => {
        // Pure relative inputs inherit the cwd prefix implicitly. Different
        // prefixes cannot arise when both sides resolve against the same cwd.
      }
      Component::RootDir => stack.clear(),
      Component::CurDir => {}
      Component::ParentDir => {
        let _ = stack.pop();
      }
      Component::Normal(normal) => stack.push(normal),
    }
  }
  Some(stack)
}

fn apply_relative_shape_to_stack<'a>(
  cwd_stack: &OsStrVec<'a>,
  shape: LexicalRelativeShape,
  relative: &'a Path,
) -> OsStrVec<'a> {
  let kept = cwd_stack.len().saturating_sub(shape.unresolved_parents);
  let mut stack = OsStrVec::with_capacity(kept + shape.surviving_normals);
  stack.extend_from_slice(&cwd_stack[..kept]);
  for normal in collect_lexical_normals(relative, shape) {
    stack.push(normal);
  }
  stack
}

fn relative_from_normal_stacks(base: &OsStrVec<'_>, target: &OsStrVec<'_>) -> Option<PathBuf> {
  let common_len = base
    .iter()
    .zip(target.iter())
    .take_while(|(from, to)| {
      #[cfg(target_family = "windows")]
      {
        from.eq_ignore_ascii_case(to)
      }
      #[cfg(not(target_family = "windows"))]
      {
        from == to
      }
    })
    .count();

  let up_len = base.len() - common_len;
  let target_suffix = &target[common_len..];
  #[cfg(target_family = "windows")]
  if target_suffix.iter().any(|component| memchr(b'/', component.as_encoded_bytes()).is_some())
    || (up_len == 0
      && target_suffix.first().is_some_and(|component| {
        !windows_standalone_relative_bytes_are_representable(component.as_encoded_bytes())
      }))
  {
    return None;
  }

  let component_count = up_len + target_suffix.len();
  let capacity = up_len * 2
    + target_suffix.iter().map(|component| component.len()).sum::<usize>()
    + component_count.saturating_sub(1);
  #[cfg(target_family = "windows")]
  {
    let mut relative = OsString::with_capacity(capacity);
    for _ in 0..up_len {
      push_windows_relative_component(&mut relative, OsStr::new(".."));
    }
    for component in target_suffix {
      push_windows_relative_component(&mut relative, component);
    }
    Some(PathBuf::from(relative))
  }
  #[cfg(not(target_family = "windows"))]
  {
    let mut relative = PathBuf::with_capacity(capacity);
    for _ in 0..up_len {
      relative.push(Component::ParentDir);
    }
    for component in target_suffix {
      relative.push(*component);
    }
    Some(relative)
  }
}

/// When both inputs are pure relative but do not share a leading-parent count
/// (or other cwd-independent shape), resolve them against one absolute cwd as
/// normal-component stacks and build a single relative result. Avoids cloning
/// cwd twice and allocating two intermediate absolute `PathBuf`s.
///
/// Callers that already hold shapes should use
/// [`relative_both_relative_via_cwd_with_shapes`] so each path is classified once.
fn try_relative_both_relative_via_cwd(target: &Path, base: &Path, cwd: &Path) -> Option<PathBuf> {
  let target_shape = classify_lexical_relative(target)?;
  let base_shape = classify_lexical_relative(base)?;
  relative_both_relative_via_cwd_with_shapes(target, target_shape, base, base_shape, cwd)
}

fn relative_both_relative_via_cwd_with_shapes(
  target: &Path,
  target_shape: LexicalRelativeShape,
  base: &Path,
  base_shape: LexicalRelativeShape,
  cwd: &Path,
) -> Option<PathBuf> {
  let cwd_stack = absolute_normal_stack(cwd)?;
  let base_resolved = apply_relative_shape_to_stack(&cwd_stack, base_shape, base);
  let target_resolved = apply_relative_shape_to_stack(&cwd_stack, target_shape, target);
  relative_from_normal_stacks(&base_resolved, &target_resolved)
}

#[cfg(target_family = "windows")]
fn classify_drive_relative(path: &Path) -> Option<(u8, LexicalRelativeShape)> {
  use std::path::Prefix;

  if path.has_root() {
    return None;
  }
  let mut components = path.components();
  let Component::Prefix(prefix) = components.next()? else {
    return None;
  };
  let Prefix::Disk(parsed_drive) = prefix.kind() else {
    return None;
  };
  let drive = windows_drive_spelling(path).unwrap_or(parsed_drive);

  let mut unresolved_parents = 0;
  let mut surviving_normals = 0;
  let mut max_normal_depth = 0;
  for component in components {
    match component {
      Component::CurDir => {}
      Component::ParentDir => {
        if surviving_normals > 0 {
          surviving_normals -= 1;
        } else {
          unresolved_parents += 1;
        }
      }
      Component::Normal(_) => {
        surviving_normals += 1;
        max_normal_depth = max_normal_depth.max(surviving_normals);
      }
      Component::Prefix(_) | Component::RootDir => return None,
    }
  }

  Some((drive, LexicalRelativeShape { unresolved_parents, surviving_normals, max_normal_depth }))
}

#[cfg(target_family = "windows")]
fn windows_drive_spelling(path: &Path) -> Option<u8> {
  let bytes = path.as_os_str().as_encoded_bytes();
  if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
    return Some(bytes[0]);
  }
  if bytes.len() >= 6
    && matches!(bytes[0], b'/' | b'\\')
    && matches!(bytes[1], b'/' | b'\\')
    && bytes[2] == b'?'
    && matches!(bytes[3], b'/' | b'\\')
    && bytes[5] == b':'
    && bytes[4].is_ascii_alphabetic()
  {
    return Some(bytes[4]);
  }
  None
}

#[cfg(target_family = "windows")]
fn push_windows_relative_component(path: &mut OsString, component: &OsStr) {
  if !path.is_empty() {
    path.push("\\");
  }
  path.push(component);
}

#[cfg(target_family = "windows")]
fn push_windows_path_component(
  path: &mut OsString,
  component: &OsStr,
  forward_slash_is_separator: bool,
) {
  let has_separator = matches!(path.as_encoded_bytes().last(), Some(b'\\'))
    || (forward_slash_is_separator && matches!(path.as_encoded_bytes().last(), Some(b'/')));
  if !path.is_empty() && !has_separator {
    path.push("\\");
  }
  path.push(component);
}

#[cfg(target_family = "windows")]
fn collect_drive_relative_normals(path: &Path, shape: LexicalRelativeShape) -> OsStrVec<'_> {
  let mut normals = OsStrVec::with_capacity(shape.max_normal_depth);
  for component in path.components().skip(1) {
    match component {
      Component::CurDir => {}
      Component::ParentDir => {
        normals.pop();
      }
      Component::Normal(normal) => normals.push(normal),
      Component::Prefix(_) | Component::RootDir => {
        unreachable!("classified drive-relative paths have one prefix and no root")
      }
    }
  }
  debug_assert_eq!(normals.len(), shape.surviving_normals);
  normals
}

#[cfg(target_family = "windows")]
fn try_relative_drive_lexically(target: &Path, base: &Path) -> Option<PathBuf> {
  let (target_drive, target_shape) = classify_drive_relative(target)?;
  let (base_drive, base_shape) = classify_drive_relative(base)?;
  if !target_drive.eq_ignore_ascii_case(&base_drive)
    || target_shape.unresolved_parents != base_shape.unresolved_parents
  {
    return None;
  }

  let target = collect_drive_relative_normals(target, target_shape);
  let base = collect_drive_relative_normals(base, base_shape);
  let common_len =
    target.iter().zip(&base).take_while(|(target, base)| target.eq_ignore_ascii_case(base)).count();
  let up_len = base.len() - common_len;
  let target_suffix = &target[common_len..];
  if up_len == 0
    && target_suffix.first().is_some_and(|component| {
      !windows_standalone_relative_bytes_are_representable(component.as_encoded_bytes())
    })
  {
    return None;
  }
  let component_count = up_len + target_suffix.len();
  let capacity = up_len * 2
    + target_suffix.iter().map(|component| component.len()).sum::<usize>()
    + component_count.saturating_sub(1);
  let mut relative = OsString::with_capacity(capacity);
  for _ in 0..up_len {
    push_windows_relative_component(&mut relative, OsStr::new(".."));
  }
  for component in target_suffix {
    push_windows_relative_component(&mut relative, component);
  }
  Some(PathBuf::from(relative))
}

#[cfg(target_family = "windows")]
fn windows_absolute_disk_drive(path: &Path) -> Option<(u8, bool)> {
  use std::path::Prefix;

  let Component::Prefix(prefix) = path.components().next()? else {
    return None;
  };
  match prefix.kind() {
    Prefix::Disk(drive) => Some((windows_drive_spelling(path).unwrap_or(drive), true)),
    Prefix::VerbatimDisk(drive) => Some((windows_drive_spelling(path).unwrap_or(drive), false)),
    _ => None,
  }
}

#[cfg(target_family = "windows")]
fn rebuild_windows_disk_path(path: &Path, drive: u8) -> Option<PathBuf> {
  let mut rebuilt = OsString::with_capacity(path.as_os_str().len().max(3));
  let prefix = [drive, b':', b'\\'];
  rebuilt.push(std::str::from_utf8(&prefix).expect("Windows drive prefix is ASCII"));
  for component in path.components() {
    match component {
      Component::Prefix(_) | Component::RootDir | Component::CurDir => {}
      Component::ParentDir | Component::Normal(_) => {
        if memchr(b'/', component.as_os_str().as_encoded_bytes()).is_some() {
          return None;
        }
        push_windows_path_component(&mut rebuilt, component.as_os_str(), true);
      }
    }
  }
  Some(PathBuf::from(rebuilt))
}

#[cfg(target_family = "windows")]
fn rebuild_windows_verbatim_disk_path(path: &Path, drive: u8) -> PathBuf {
  let mut rebuilt = OsString::with_capacity(path.as_os_str().len().max(7));
  let prefix = [b'\\', b'\\', b'?', b'\\', drive, b':', b'\\'];
  rebuilt.push(std::str::from_utf8(&prefix).expect("Windows verbatim drive prefix is ASCII"));
  for component in path.components() {
    match component {
      Component::Prefix(_) | Component::RootDir | Component::CurDir => {}
      Component::ParentDir | Component::Normal(_) => {
        push_windows_path_component(&mut rebuilt, component.as_os_str(), false);
      }
    }
  }
  PathBuf::from(rebuilt)
}

#[cfg(target_family = "windows")]
fn preserve_windows_drive_spelling(path: PathBuf, drive: u8) -> PathBuf {
  match windows_absolute_disk_drive(&path) {
    Some((actual, true)) if actual == drive => path,
    Some((actual, true)) if actual.eq_ignore_ascii_case(&drive) => {
      rebuild_windows_disk_path(&path, drive).unwrap_or(path)
    }
    Some((actual, false)) if actual.eq_ignore_ascii_case(&drive) => {
      rebuild_windows_verbatim_disk_path(&path, drive)
    }
    _ => path,
  }
}

#[cfg(target_family = "windows")]
fn absolutize_drive_relative_with<P>(path: &Path, cwd: P, drive: u8) -> Cow<'_, Path>
where
  P: AsRef<Path> + Into<PathBuf>,
{
  let Some((cwd_drive, ordinary_disk)) = windows_absolute_disk_drive(cwd.as_ref()) else {
    return normalize_for_resolution(path);
  };
  if !cwd_drive.eq_ignore_ascii_case(&drive) {
    return normalize_for_resolution(path);
  }

  let resolved = if cwd_drive == drive {
    cwd.into()
  } else if ordinary_disk {
    rebuild_windows_disk_path(cwd.as_ref(), drive)
      .expect("ordinary Windows disk components contain no literal forward slash")
  } else {
    rebuild_windows_verbatim_disk_path(cwd.as_ref(), drive)
  };
  let mut resolved = resolved.into_os_string();
  for component in path.components().skip(1) {
    push_windows_path_component(&mut resolved, component.as_os_str(), ordinary_disk);
  }
  Cow::Owned(normalize_owned_for_resolution(PathBuf::from(resolved)))
}

/// Normalize an owned buffer, reusing its allocation whenever capacity is enough.
///
/// Dirty inputs almost always shrink (`.` / `..` / duplicate separators removed),
/// so join → `into_normalized` and `absolutize_with` (cwd push then normalize)
/// avoid a second heap allocation for the rebuild on typical path lengths.
fn normalize_owned_path_buf_with(path: PathBuf, trailing: TrailingSeparator) -> PathBuf {
  if !needs_normalization(&path, trailing) {
    return path;
  }

  // Long paths: keep the previous one-allocation rebuild via `normalize_inner`.
  if path.as_os_str().len() > OWNED_NORMALIZE_STACK_ARENA {
    return normalize_owned_path_buf_via_inner(path, trailing);
  }

  normalize_owned_path_buf_reusing(path, trailing)
}

fn normalize_owned_path_buf_via_inner(mut path: PathBuf, trailing: TrailingSeparator) -> PathBuf {
  let preserve_trailing = trailing == TrailingSeparator::Preserve && has_trailing_separator(&path);
  let drive = {
    #[cfg(target_family = "windows")]
    {
      windows_drive_spelling(&path)
    }
    #[cfg(not(target_family = "windows"))]
    {
      None
    }
  };
  match normalize_inner(
    path.components().peekable(),
    path.as_os_str().len(),
    preserve_trailing,
    drive,
  ) {
    Cow::Borrowed(normalized) if std::ptr::eq(normalized, path.as_path()) => path,
    Cow::Borrowed(normalized) if normalized.as_os_str() == OsStr::new(".") => {
      path.clear();
      path.push(".");
      path
    }
    Cow::Borrowed(normalized) => normalized.to_path_buf(),
    Cow::Owned(normalized) => normalized,
  }
}

fn normalize_owned_path_buf_reusing(path: PathBuf, trailing: TrailingSeparator) -> PathBuf {
  let preserve_trailing = trailing == TrailingSeparator::Preserve && has_trailing_separator(&path);
  #[cfg(target_family = "windows")]
  let original_len = path.as_os_str().len();
  #[cfg(target_family = "windows")]
  let drive_spelling = windows_drive_spelling(&path);

  let mut arena = [0u8; OWNED_NORMALIZE_STACK_ARENA];
  let mut arena_len = 0usize;
  let mut stack: [(u16, u16); 24] = [(0, 0); 24];
  let mut stack_len = 0usize;
  let mut has_root = false;
  let mut leading_parents = 0usize;

  #[cfg(target_family = "windows")]
  let mut prefix_range: Option<(u16, u16)> = None;
  #[cfg(target_family = "windows")]
  let mut prefix_only_suffix: Option<u8> = None;
  #[cfg(target_family = "windows")]
  let mut prefix_root_is_optional = false;

  let push_arena = |arena: &mut [u8; OWNED_NORMALIZE_STACK_ARENA],
                    arena_len: &mut usize,
                    bytes: &[u8]|
   -> Option<(u16, u16)> {
    if *arena_len + bytes.len() > OWNED_NORMALIZE_STACK_ARENA {
      return None;
    }
    let start = *arena_len;
    arena[start..start + bytes.len()].copy_from_slice(bytes);
    *arena_len += bytes.len();
    Some((start as u16, *arena_len as u16))
  };

  for component in path.components() {
    match component {
      #[cfg(target_family = "windows")]
      Component::Prefix(p) => {
        let start = arena_len;
        let (suffix, optional_root, extra): (Option<u8>, bool, SmallVec<[u8; 64]>) = match p.kind()
        {
          std::path::Prefix::VerbatimDisk(drive) => {
            let mut extra = SmallVec::new();
            extra.extend_from_slice(b"\\\\?\\");
            extra.push(drive_spelling.unwrap_or(drive));
            extra.push(b':');
            (None, false, extra)
          }
          std::path::Prefix::DeviceNS(device) => {
            let mut extra = SmallVec::new();
            extra.extend_from_slice(b"\\\\.\\");
            extra.extend_from_slice(device.as_encoded_bytes());
            (None, true, extra)
          }
          std::path::Prefix::UNC(server, share) => {
            let mut extra = SmallVec::new();
            extra.extend_from_slice(b"\\\\");
            extra.extend_from_slice(server.as_encoded_bytes());
            extra.push(b'\\');
            extra.extend_from_slice(share.as_encoded_bytes());
            (Some(b'\\'), false, extra)
          }
          std::path::Prefix::Disk(drive) => {
            let mut extra = SmallVec::new();
            extra.push(drive_spelling.unwrap_or(drive));
            extra.push(b':');
            (Some(b'.'), false, extra)
          }
          std::path::Prefix::Verbatim(_) | std::path::Prefix::VerbatimUNC(_, _) => {
            let mut extra = SmallVec::new();
            extra.extend_from_slice(p.as_os_str().as_encoded_bytes());
            (None, true, extra)
          }
        };
        if push_arena(&mut arena, &mut arena_len, &extra).is_none() {
          return normalize_owned_path_buf_via_inner(path, trailing);
        }
        prefix_only_suffix = suffix;
        prefix_root_is_optional = optional_root;
        prefix_range = Some((start as u16, arena_len as u16));
      }
      #[cfg(not(target_family = "windows"))]
      Component::Prefix(_) => unreachable!("prefix components only exist on Windows"),
      Component::RootDir => {
        has_root = true;
        stack_len = 0;
        leading_parents = 0;
      }
      Component::CurDir => {}
      Component::ParentDir => {
        if stack_len > 0 {
          stack_len -= 1;
        } else if !has_root {
          leading_parents += 1;
        }
      }
      Component::Normal(normal) => {
        if stack_len >= stack.len() {
          return normalize_owned_path_buf_via_inner(path, trailing);
        }
        let Some(range) = push_arena(&mut arena, &mut arena_len, normal.as_encoded_bytes()) else {
          return normalize_owned_path_buf_via_inner(path, trailing);
        };
        stack[stack_len] = range;
        stack_len += 1;
      }
    }
  }

  let sep_byte = std::path::MAIN_SEPARATOR as u8;
  let mut buf = path.into_os_string().into_encoded_bytes();
  buf.clear();

  #[cfg(target_family = "windows")]
  let prefix_len = {
    if let Some((start, end)) = prefix_range {
      buf.extend_from_slice(&arena[start as usize..end as usize]);
    }
    buf.len()
  };
  #[cfg(not(target_family = "windows"))]
  let _prefix_len = 0usize;

  if has_root {
    #[cfg(target_family = "windows")]
    {
      let prefix_only_input = prefix_root_is_optional && prefix_len == original_len;
      if !prefix_only_input {
        buf.push(sep_byte);
      }
    }
    #[cfg(not(target_family = "windows"))]
    {
      buf.push(sep_byte);
    }
  }

  let mut need_sep = false;
  for _ in 0..leading_parents {
    if need_sep {
      buf.push(sep_byte);
    }
    buf.extend_from_slice(b"..");
    need_sep = true;
  }
  for range in stack.iter().take(stack_len) {
    if need_sep {
      buf.push(sep_byte);
    }
    buf.extend_from_slice(&arena[range.0 as usize..range.1 as usize]);
    need_sep = true;
  }

  #[cfg(target_family = "windows")]
  if prefix_root_is_optional && stack_len == 0 && leading_parents == 0 && !preserve_trailing {
    buf.truncate(prefix_len);
  }

  #[cfg(target_family = "windows")]
  if prefix_len == 0 && !has_root && !windows_standalone_relative_bytes_are_representable(&buf) {
    let len = buf.len();
    buf.reserve(2);
    buf.resize(len + 2, 0);
    buf.copy_within(0..len, 2);
    buf[0] = b'.';
    buf[1] = sep_byte;
  }

  if buf.is_empty() {
    if preserve_trailing {
      buf.extend_from_slice(b".");
      buf.push(sep_byte);
    } else {
      buf.extend_from_slice(b".");
    }
  } else {
    #[cfg(target_family = "windows")]
    if buf.len() == prefix_len
      && prefix_len > 0
      && let Some(suffix) = prefix_only_suffix
    {
      buf.push(suffix);
    }

    if preserve_trailing && buf.last() != Some(&sep_byte) {
      buf.push(sep_byte);
    }
  }

  // SAFETY: buf is built from OsStr component bytes plus ASCII separators / dots.
  PathBuf::from(unsafe { OsString::from_encoded_bytes_unchecked(buf) })
}

pub(crate) fn normalize_owned_path_buf(path: PathBuf) -> PathBuf {
  normalize_owned_path_buf_with(path, TrailingSeparator::Preserve)
}

fn normalize_owned_for_resolution(path: PathBuf) -> PathBuf {
  normalize_owned_path_buf_with(path, TrailingSeparator::Strip)
}

fn normalize_path(path: &Path, trailing: TrailingSeparator) -> Cow<'_, Path> {
  if !needs_normalization(path, trailing) {
    return Cow::Borrowed(path);
  }
  normalize_inner(
    path.components().peekable(),
    path.as_os_str().len(),
    trailing == TrailingSeparator::Preserve && has_trailing_separator(path),
    {
      #[cfg(target_family = "windows")]
      {
        windows_drive_spelling(path)
      }
      #[cfg(not(target_family = "windows"))]
      {
        None
      }
    },
  )
}

fn normalize_for_resolution(path: &Path) -> Cow<'_, Path> {
  normalize_path(path, TrailingSeparator::Strip)
}

#[inline]
fn has_trailing_separator(path: &Path) -> bool {
  let Some(last) = path.as_os_str().as_encoded_bytes().last() else {
    return false;
  };
  if *last == std::path::MAIN_SEPARATOR as u8 {
    return true;
  }

  #[cfg(target_family = "windows")]
  {
    *last == b'/'
      && !matches!(
        path.components().next(),
        Some(Component::Prefix(prefix))
          if windows_prefix_is_verbatim(prefix.kind())
      )
  }
  #[cfg(not(target_family = "windows"))]
  {
    false
  }
}

fn replace_main_separator_in_owned(mut string: String) -> String {
  if std::path::MAIN_SEPARATOR == '/' {
    string
  } else {
    let mut offset = 0;
    while let Some(position) = memchr(std::path::MAIN_SEPARATOR as u8, &string.as_bytes()[offset..])
    {
      let separator = offset + position;
      string.replace_range(separator..=separator, "/");
      offset = separator + 1;
    }
    string
  }
}

pub(crate) fn try_path_buf_into_slash(path: PathBuf) -> Result<String, PathBuf> {
  match path.into_os_string().into_string() {
    Ok(string) => Ok(replace_main_separator_in_owned(string)),
    Err(path) => Err(PathBuf::from(path)),
  }
}

pub(crate) fn path_buf_into_slash(path: PathBuf) -> String {
  try_path_buf_into_slash(path).expect("path is not valid Unicode")
}

pub(crate) fn path_buf_into_slash_lossy(path: PathBuf) -> String {
  match try_path_buf_into_slash(path) {
    Ok(string) => string,
    Err(path) => replace_main_separator_in_owned(path.to_string_lossy().into_owned()),
  }
}

#[cfg(target_family = "windows")]
fn windows_absolute_parts(path: &Path) -> Option<(std::path::Prefix<'_>, &Path)> {
  let mut components = path.components();
  let Component::Prefix(prefix) = components.next()? else {
    return None;
  };
  if matches!(components.clone().next(), Some(Component::RootDir)) {
    components.next();
  }
  Some((prefix.kind(), components.as_path()))
}

#[cfg(target_family = "windows")]
fn windows_prefix_is_verbatim(prefix: std::path::Prefix<'_>) -> bool {
  matches!(
    prefix,
    std::path::Prefix::Verbatim(_)
      | std::path::Prefix::VerbatimDisk(_)
      | std::path::Prefix::VerbatimUNC(_, _)
  )
}

#[cfg(target_family = "windows")]
fn windows_standalone_relative_is_representable(path: &str) -> bool {
  windows_standalone_relative_bytes_are_representable(path.as_bytes())
}

#[cfg(target_family = "windows")]
fn windows_standalone_relative_bytes_are_representable(path: &[u8]) -> bool {
  !matches!(path.first(), Some(b'/' | b'\\'))
    && !(path.len() >= 2 && path[0].is_ascii_alphabetic() && path[1] == b':')
}

#[cfg(target_family = "windows")]
fn windows_relative_component_has_literal_slash(component: &Component<'_>) -> bool {
  let Component::Normal(component) = component else {
    return false;
  };
  memchr(b'/', component.as_encoded_bytes()).is_some()
}

#[cfg(target_family = "windows")]
fn windows_native_relative_input_is_clean(path: &str) -> bool {
  if memchr(b'/', path.as_bytes()).is_some() {
    return false;
  }

  let path = path.trim_end_matches('\\');
  let mut offset = 0;
  while offset < path.len() {
    let end = memchr(b'\\', &path.as_bytes()[offset..])
      .map(|position| offset + position)
      .unwrap_or(path.len());
    let component = &path[offset..end];
    if component.is_empty() || component == "." || component == ".." {
      return false;
    }
    offset = end.saturating_add(1);
  }
  true
}

#[cfg(target_family = "windows")]
fn relative_windows_native_fast<'a>(target: &'a str, base: &str) -> Option<RelativeOutcome<'a>> {
  if !windows_native_relative_input_is_clean(target)
    || !windows_native_relative_input_is_clean(base)
  {
    return None;
  }

  let target = target.trim_end_matches('\\');
  let base = base.trim_end_matches('\\');
  let common_byte_len = target
    .as_bytes()
    .iter()
    .zip(base.as_bytes())
    .take_while(|(target, base)| target.eq_ignore_ascii_case(base))
    .count();
  let at_boundary = (common_byte_len == target.len() && common_byte_len == base.len())
    || (common_byte_len == target.len() && base.as_bytes().get(common_byte_len) == Some(&b'\\'))
    || (common_byte_len == base.len() && target.as_bytes().get(common_byte_len) == Some(&b'\\'));
  let common_prefix = if at_boundary {
    common_byte_len
  } else {
    memrchr(b'\\', &target.as_bytes()[..common_byte_len]).unwrap_or(0)
  };

  let base_remaining = &base.as_bytes()[common_prefix..];
  let mut ups = 0usize;
  let mut offset = 0;
  while offset < base_remaining.len() {
    if base_remaining[offset] == b'\\' {
      offset += 1;
      continue;
    }
    ups += 1;
    offset = memchr(b'\\', &base_remaining[offset..])
      .map(|position| offset + position + 1)
      .unwrap_or(base_remaining.len());
  }

  let target_suffix = target[common_prefix..].trim_start_matches('\\');
  if ups == 0 {
    if !windows_standalone_relative_is_representable(target_suffix) {
      return None;
    }
    return Some(RelativeOutcome::BorrowedNative(Path::new(target_suffix)));
  }

  let mut relative = String::with_capacity(ups * 3 + target_suffix.len());
  for _ in 0..ups {
    if !relative.is_empty() {
      relative.push('\\');
    }
    relative.push_str("..");
  }
  if !target_suffix.is_empty() {
    relative.push('\\');
    relative.push_str(target_suffix);
  }
  Some(RelativeOutcome::Native(PathBuf::from(relative)))
}

#[cfg(target_family = "windows")]
fn try_relative_windows_absolute<'a>(
  target_path: &'a Path,
  base_path: &Path,
) -> Option<RelativeOutcome<'a>> {
  let (target_prefix, target_rest) = windows_absolute_parts(target_path)?;
  let (base_prefix, base_rest) = windows_absolute_parts(base_path)?;
  if !windows_prefixes_eq_ignore_ascii_case(target_prefix, base_prefix) {
    return Some(RelativeOutcome::Native(normalize_for_resolution(target_path).into_owned()));
  }

  // A forward slash is a literal byte inside a verbatim path component. The
  // normalized-string fallback would reinterpret it as a separator, so let
  // the component fallback retain the native `std::path` meaning instead.
  if windows_prefix_is_verbatim(target_prefix)
    && (memchr(b'/', target_rest.as_os_str().as_encoded_bytes()).is_some()
      || memchr(b'/', base_rest.as_os_str().as_encoded_bytes()).is_some())
  {
    return None;
  }

  let (target_str, base_str) = (target_rest.to_str()?, base_rest.to_str()?);
  if let Some(outcome) = relative_windows_native_fast(target_str, base_str) {
    return Some(outcome);
  }

  // Dirty or mixed-separator paths retain the established normalized-string
  // fallback. Its temporary allocations stay off the canonical Rolldown path.
  let target_fwd = normalize_backslash_cow(target_str);
  let base_fwd = normalize_backslash_cow(base_str);
  let relative = relative_str(&target_fwd, &base_fwd);
  if !windows_standalone_relative_is_representable(&relative) {
    return None;
  }
  Some(match relative {
    Cow::Borrowed(relative) => {
      let target_without_trailing = target_str.trim_end_matches(['/', '\\']);
      debug_assert!(relative.len() <= target_without_trailing.len());
      let original_relative =
        &target_without_trailing[target_without_trailing.len() - relative.len()..];
      if memchr(b'/', original_relative.as_bytes()).is_none() {
        RelativeOutcome::BorrowedNative(Path::new(original_relative))
      } else {
        RelativeOutcome::Slash(relative.to_owned())
      }
    }
    Cow::Owned(relative) => RelativeOutcome::Slash(relative),
  })
}

#[cfg(target_family = "windows")]
fn try_relative_windows_root_lexically(target: &Path, base: &Path) -> Option<PathBuf> {
  if !matches!(target.components().next(), Some(Component::RootDir))
    || !matches!(base.components().next(), Some(Component::RootDir))
  {
    return None;
  }

  let target = normalize_for_resolution(target);
  let base = normalize_for_resolution(base);
  Some(relative_from_resolved(base, target).into_path_buf())
}

fn relative_without_cwd<'a>(
  target_path: &'a Path,
  base_path: &Path,
) -> Option<RelativeOutcome<'a>> {
  // Fast path: absolute inputs do not need cwd state. Unix can scan the UTF-8
  // spelling directly. Windows first compares borrowed prefix components, then
  // scans canonical native separators without allocating normalized copies.
  #[cfg(target_family = "windows")]
  if target_path.is_absolute()
    && base_path.is_absolute()
    && let Some(outcome) = try_relative_windows_absolute(target_path, base_path)
  {
    return Some(outcome);
  }

  #[cfg(not(target_family = "windows"))]
  if target_path.is_absolute()
    && base_path.is_absolute()
    && let (Some(target_str), Some(base_str)) = (target_path.to_str(), base_path.to_str())
  {
    return Some(match relative_str(target_str, base_str) {
      Cow::Borrowed(relative) => RelativeOutcome::BorrowedNative(Path::new(relative)),
      Cow::Owned(relative) => RelativeOutcome::Slash(relative),
    });
  }

  #[cfg(target_family = "windows")]
  if let Some(relative) = try_relative_windows_root_lexically(target_path, base_path) {
    return Some(RelativeOutcome::Native(relative));
  }

  #[cfg(target_family = "windows")]
  if let Some(relative) = try_relative_drive_lexically(target_path, base_path) {
    return Some(RelativeOutcome::Native(relative));
  }

  // Plain relative paths with the same number of unresolved leading parents
  // resolve from the same cwd ancestor. Their relative path is therefore
  // independent of the cwd itself.
  if !target_path.has_root()
    && !base_path.has_root()
    && let Some(relative) = try_relative_lexically(target_path, base_path)
  {
    return Some(RelativeOutcome::Native(relative));
  }

  None
}

/// Build a relative outcome from already-resolved absolute paths.
///
/// Takes [`Cow`] so Windows branches that must return the absolute target can
/// move an owned buffer instead of cloning it (`into_owned` is free on
/// [`Cow::Owned`]). Clean borrowed bases stay borrowed for the common-prefix scan.
fn relative_from_resolved(base: Cow<'_, Path>, target: Cow<'_, Path>) -> RelativeOutcome<'static> {
  #[cfg(target_family = "windows")]
  if windows_paths_have_different_prefixes(base.as_ref(), target.as_ref()) {
    return RelativeOutcome::Native(target.into_owned());
  }

  if base.as_ref() == target.as_ref() {
    return RelativeOutcome::Native(PathBuf::new());
  }

  let filter_fn = |component: &Component| {
    matches!(component, Component::Normal(_) | Component::Prefix(_) | Component::RootDir)
  };
  let base_components = base.components().filter(filter_fn);
  let target_components = target.components().filter(filter_fn);
  let common_len = base_components
    .clone()
    .zip(target_components.clone())
    .take_while(|(from, to)| {
      #[cfg(target_family = "windows")]
      {
        windows_components_eq_ignore_ascii_case(from, to)
      }
      #[cfg(not(target_family = "windows"))]
      {
        from == to
      }
    })
    .count();
  let up_len = base_components.count().saturating_sub(common_len);
  #[cfg(target_family = "windows")]
  {
    let target_suffix = target_components.clone().skip(common_len);
    if target_suffix
      .clone()
      .any(|component| windows_relative_component_has_literal_slash(&component))
      || (up_len == 0
        && target_suffix.clone().next().is_some_and(|component| {
          !windows_standalone_relative_bytes_are_representable(
            component.as_os_str().as_encoded_bytes(),
          )
        }))
    {
      return RelativeOutcome::Native(target.into_owned());
    }

    let suffix_count = target_suffix.clone().count();
    let component_count = up_len + suffix_count;
    let capacity = up_len * 2
      + target_suffix.clone().map(|component| component.as_os_str().len()).sum::<usize>()
      + component_count.saturating_sub(1);
    let mut relative = OsString::with_capacity(capacity);
    for _ in 0..up_len {
      push_windows_relative_component(&mut relative, OsStr::new(".."));
    }
    for component in target_suffix {
      push_windows_relative_component(&mut relative, component.as_os_str());
    }
    RelativeOutcome::Native(PathBuf::from(relative))
  }
  #[cfg(not(target_family = "windows"))]
  let relative =
    (0..up_len).map(|_| Component::ParentDir).chain(target_components.skip(common_len)).collect();
  #[cfg(not(target_family = "windows"))]
  RelativeOutcome::Native(relative)
}

fn try_relative_outcome<'a>(
  target_path: &'a Path,
  base_path: &Path,
) -> io::Result<RelativeOutcome<'a>> {
  if let Some(outcome) = relative_without_cwd(target_path, base_path) {
    return Ok(outcome);
  }

  // Pure lexical relative pairs only (no prefix/root). Windows drive-relative
  // inputs are `!has_root()` but carry a Prefix — they must keep try_absolutize
  // so ambient relative uses per-drive `std::path::absolute`, not absolutize_with
  // against the process cwd. Classify once; reuse shapes for the stack resolve.
  if let (Some(target_shape), Some(base_shape)) =
    (classify_lexical_relative(target_path), classify_lexical_relative(base_path))
  {
    let cwd = try_get_current_dir()?;
    if let Some(relative) = relative_both_relative_via_cwd_with_shapes(
      target_path,
      target_shape,
      base_path,
      base_shape,
      cwd.as_ref(),
    ) {
      return Ok(RelativeOutcome::Native(relative));
    }
    let base = base_path.absolutize_with(cwd.as_ref());
    let target = target_path.absolutize_with(cwd.as_ref());
    return Ok(relative_from_resolved(base, target));
  }

  // Slow path: avoid current_dir() for already-absolute paths. Windows
  // drive-relative receivers still take try_absolutize here. Keep Cow so a
  // clean absolute base is not cloned solely to compute relative against a
  // dirty/invalid target, and so Windows different-prefix returns can move an
  // already-owned absolute target without a second clone.
  let base = if base_path.is_absolute() {
    normalize_for_resolution(base_path)
  } else {
    base_path.try_absolutize()?
  };
  let target = if target_path.is_absolute() {
    normalize_for_resolution(target_path)
  } else {
    target_path.try_absolutize()?
  };

  Ok(relative_from_resolved(base, target))
}

fn relative_outcome_with<'a, P>(
  target_path: &'a Path,
  base_path: &Path,
  cwd: P,
) -> RelativeOutcome<'a>
where
  P: AsRef<Path> + Into<PathBuf>,
{
  if let Some(outcome) = relative_without_cwd(target_path, base_path) {
    return outcome;
  }

  assert!(cwd.as_ref().is_absolute(), "explicit current directory must be absolute");

  // Same pure-lexical gate as ambient relative. try_relative_both_relative_via_cwd
  // already requires classification; do not treat every !has_root path as pure
  // relative (Windows drive-relative keeps absolutize_with / drive rules below).
  if let Some(relative) = try_relative_both_relative_via_cwd(target_path, base_path, cwd.as_ref()) {
    return RelativeOutcome::Native(relative);
  }

  let base = if base_path.is_absolute() {
    normalize_for_resolution(base_path)
  } else {
    base_path.absolutize_with(cwd.as_ref())
  };
  let target = if target_path.is_absolute() {
    normalize_for_resolution(target_path)
  } else {
    target_path.absolutize_with(cwd)
  };

  if !base.is_absolute() || !target.is_absolute() {
    return RelativeOutcome::Native(normalize_for_resolution(target.as_ref()).into_owned());
  }

  relative_from_resolved(base, target)
}

impl SugarPath for Path {
  fn normalize(&self) -> Cow<'_, Path> {
    normalize_path(self, TrailingSeparator::Preserve)
  }

  fn absolutize(&self) -> Cow<'_, Path> {
    self.try_absolutize().expect("failed to resolve path against the current directory")
  }

  fn try_absolutize(&self) -> io::Result<Cow<'_, Path>> {
    if self.is_absolute() {
      return Ok(normalize_for_resolution(self));
    }

    #[cfg(target_family = "windows")]
    if let Some((drive, _)) = classify_drive_relative(self) {
      let absolute = std::path::absolute(self)?;
      return Ok(Cow::Owned(normalize_owned_for_resolution(preserve_windows_drive_spelling(
        absolute, drive,
      ))));
    }

    let cwd = try_get_current_dir()?;
    Ok(self.absolutize_with(cwd))
  }

  fn absolutize_with(&self, cwd: impl AsRef<Path> + Into<PathBuf>) -> Cow<'_, Path> {
    if self.is_absolute() {
      return normalize_for_resolution(self);
    }

    assert!(cwd.as_ref().is_absolute(), "explicit current directory must be absolute");

    #[cfg(target_family = "windows")]
    if let Some((drive, _)) = classify_drive_relative(self) {
      return absolutize_drive_relative_with(self, cwd, drive);
    }

    let mut resolved: PathBuf = cwd.into();
    // Grow once for the relative suffix so push + dirty normalize reuse do not
    // thrash capacity on the common cwd+module path shape.
    resolved.as_mut_os_string().reserve(self.as_os_str().len().saturating_add(1));
    resolved.push(self);
    Cow::Owned(normalize_owned_for_resolution(resolved))
  }

  fn relative(&self, base: impl AsRef<Path>) -> Cow<'_, Path> {
    self.try_relative(base).expect("failed to resolve relative paths against the current directory")
  }

  fn try_relative(&self, base: impl AsRef<Path>) -> io::Result<Cow<'_, Path>> {
    try_relative_outcome(self, base.as_ref()).map(RelativeOutcome::into_cow_path)
  }

  fn relative_with(
    &self,
    base: impl AsRef<Path>,
    cwd: impl AsRef<Path> + Into<PathBuf>,
  ) -> Cow<'_, Path> {
    relative_outcome_with(self, base.as_ref(), cwd).into_cow_path()
  }

  fn to_slash(&self) -> Cow<'_, str> {
    self.try_to_slash().expect("path is not valid Unicode")
  }

  fn try_to_slash(&self) -> Option<Cow<'_, str>> {
    if std::path::MAIN_SEPARATOR == '/' {
      self.to_str().map(Cow::Borrowed)
    } else {
      self.to_str().map(|s| match replace_main_separator(s) {
        Some(replaced) => Cow::Owned(replaced),
        None => Cow::Borrowed(s),
      })
    }
  }

  fn to_slash_lossy(&self) -> Cow<'_, str> {
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
#[inline(never)]
#[cfg(not(target_family = "windows"))]
fn needs_normalization(path: &Path, trailing: TrailingSeparator) -> bool {
  // Keep Cygwin's existing Unicode validation path. Its native path parser is
  // not covered by the non-Windows encoded-byte fast path.
  #[cfg(target_os = "cygwin")]
  let Some(s) = path.to_str() else {
    return true;
  };
  #[cfg(target_os = "cygwin")]
  let bytes = s.as_bytes();
  #[cfg(not(target_os = "cygwin"))]
  let bytes = path.as_os_str().as_encoded_bytes();
  // OsStr's encoded representation is self-synchronizing and preserves ASCII,
  // so native separators and dot components cannot hide inside another unit.
  let separator = std::path::MAIN_SEPARATOR as u8;
  if bytes.is_empty() {
    return true;
  }
  if bytes == b"." || (trailing == TrailingSeparator::Preserve && bytes == [b'.', separator]) {
    return false;
  }
  // A leading `.` needs normalization. A leading run of `..` can be clean when
  // every remaining component is normal (`...` and `.foo` are normal names).
  if bytes[0] == b'.' {
    if bytes.len() == 1 || bytes[1] == separator {
      return true;
    }
    if bytes[1] == b'.' && (bytes.len() == 2 || bytes[2] == separator) {
      return !leading_parent_path_is_normalized(
        bytes,
        separator,
        trailing == TrailingSeparator::Preserve,
      );
    }
  }
  // Trailing separator (unless the path is exactly the root separator).
  if trailing == TrailingSeparator::Strip && bytes.len() > 1 && bytes[bytes.len() - 1] == separator
  {
    return true;
  }
  // Scan for duplicate separators and dot components.
  let mut offset = 0;
  while let Some(pos) = memchr(separator, &bytes[offset..]) {
    let slash = offset + pos;
    let next = slash + 1;
    if next < bytes.len() {
      let b = bytes[next];
      // `//` — consecutive slashes
      if b == separator {
        return true;
      }
      // `/.` — could be `/.` or `/..`
      if b == b'.' {
        let after_dot = next + 1;
        // "/." at end or "/./"
        if after_dot >= bytes.len() || bytes[after_dot] == separator {
          return true;
        }
        // "/.." at end or "/../"
        if bytes[after_dot] == b'.'
          && (after_dot + 1 >= bytes.len() || bytes[after_dot + 1] == separator)
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
#[inline(never)]
#[cfg(target_family = "windows")]
fn needs_normalization(path: &Path, trailing: TrailingSeparator) -> bool {
  // OsStr's encoded representation is self-synchronizing and preserves ASCII,
  // so separators and dot components cannot hide inside an encoded unit.
  let bytes = path.as_os_str().as_encoded_bytes();
  if bytes.is_empty() {
    return true;
  }
  if bytes == b"." || (trailing == TrailingSeparator::Preserve && bytes == b".\\") {
    return false;
  }
  // Any forward slash means normalization is needed (gets converted to `\`)
  if memchr(b'/', bytes).is_some() {
    return true;
  }
  // UNC prefix `\\` at start — always bail out to normalizer
  if bytes.len() >= 2 && bytes[0] == b'\\' && bytes[1] == b'\\' {
    return true;
  }
  // A bare drive `X:` normalizes to `X:.`. Other drive-relative paths keep
  // their drive spelling and lexical form.
  if bytes.len() == 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
    return true; // bare `C:`
  }
  // `C:.` and `C:.\` are canonical drive-relative current-directory
  // spellings, but the same component before another segment is redundant.
  if bytes.len() > 4
    && bytes[1] == b':'
    && bytes[0].is_ascii_alphabetic()
    && bytes[2] == b'.'
    && bytes[3] == b'\\'
  {
    return true;
  }
  // A leading `.` needs normalization. A leading run of `..` can be clean when
  // every remaining component is normal (`...` and `.foo` are normal names).
  if bytes[0] == b'.' {
    if bytes.len() == 1 || bytes[1] == b'\\' {
      return true;
    }
    if bytes[1] == b'.' && (bytes.len() == 2 || bytes[2] == b'\\') {
      return !leading_parent_path_is_normalized(
        bytes,
        b'\\',
        trailing == TrailingSeparator::Preserve,
      );
    }
  }
  // Trailing `\` (unless path is `\` alone or `X:\`)
  if trailing == TrailingSeparator::Strip && bytes[bytes.len() - 1] == b'\\' {
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

/// Return whether an unprefixed path starting with a `..` component is already
/// in the exact spelling produced by `normalize_inner`.
///
/// A canonical path may contain one or more leading `..` components followed
/// only by normal components. Empty components, `.`, a `..` after any normal
/// component, and a trailing separator all require normalization.
#[inline]
fn leading_parent_path_is_normalized(bytes: &[u8], separator: u8, preserve_trailing: bool) -> bool {
  debug_assert!(bytes == b".." || bytes.starts_with(&[b'.', b'.', separator]));

  let mut offset = 0;
  let mut saw_normal = false;
  loop {
    let end =
      memchr(separator, &bytes[offset..]).map(|position| offset + position).unwrap_or(bytes.len());
    let component = &bytes[offset..end];

    if component == b".." {
      if saw_normal {
        return false;
      }
    } else if component.is_empty() || component == b"." {
      return false;
    } else {
      saw_normal = true;
    }

    if end == bytes.len() {
      return true;
    }
    offset = end + 1;
    if offset == bytes.len() {
      return preserve_trailing;
    }
  }
}

#[inline]
fn normalize_inner<'a>(
  mut components: Peekable<impl Iterator<Item = Component<'a>>>,
  hint_cap: usize,
  preserve_trailing: bool,
  _drive_spelling: Option<u8>,
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
  let prefix_only_suffix: Option<u8>;
  #[cfg(target_family = "windows")]
  let prefix_root_is_optional: bool;
  #[cfg(target_family = "windows")]
  {
    if let Some(Component::Prefix(p)) = components.peek() {
      let (suffix, optional_root) = match p.kind() {
        std::path::Prefix::VerbatimDisk(drive) => {
          buf.extend_from_slice(b"\\\\?\\");
          buf.push(_drive_spelling.unwrap_or(drive));
          buf.push(b':');
          // `\\?\C:` has no RootDir component. A slash would add one, while
          // a dot would make Rust parse the whole prefix as generic Verbatim.
          (None, false)
        }
        std::path::Prefix::DeviceNS(device) => {
          buf.extend_from_slice(b"\\\\.\\");
          buf.extend_from_slice(device.as_encoded_bytes());
          (None, true)
        }
        std::path::Prefix::UNC(server, share) => {
          buf.extend_from_slice(b"\\\\");
          buf.extend_from_slice(server.as_encoded_bytes());
          buf.push(b'\\');
          buf.extend_from_slice(share.as_encoded_bytes());
          (Some(b'\\'), false)
        }
        std::path::Prefix::Disk(drive) => {
          buf.push(_drive_spelling.unwrap_or(drive));
          buf.push(b':');
          (Some(b'.'), false)
        }
        std::path::Prefix::Verbatim(_) | std::path::Prefix::VerbatimUNC(_, _) => {
          buf.extend_from_slice(p.as_os_str().as_encoded_bytes());
          (None, true)
        }
      };
      prefix_only_suffix = suffix;
      prefix_root_is_optional = optional_root;
      components.next();
    } else {
      prefix_only_suffix = None;
      prefix_root_is_optional = false;
    }
    prefix_len = buf.len();
  }

  // --- RootDir ---
  if matches!(components.peek(), Some(Component::RootDir)) {
    #[cfg(target_family = "windows")]
    let prefix_has_synthetic_root = buf.len() == hint_cap && prefix_root_is_optional;
    #[cfg(not(target_family = "windows"))]
    let prefix_has_synthetic_root = false;
    if !prefix_has_synthetic_root {
      buf.push(sep_byte);
    }
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

  #[cfg(target_family = "windows")]
  if prefix_root_is_optional && depth == 0 && !preserve_trailing {
    buf.truncate(prefix_len);
  }

  // A normal component such as `C:foo` is only a drive prefix at the start of
  // a standalone Windows path. Keep the minimal `.\` spelling when removing
  // earlier lexical components would otherwise change that component's type.
  #[cfg(target_family = "windows")]
  if prefix_len == 0 && !has_root && !windows_standalone_relative_bytes_are_representable(&buf) {
    let len = buf.len();
    buf.reserve(2);
    buf.resize(len + 2, 0);
    buf.copy_within(0..len, 2);
    buf[0] = b'.';
    buf[1] = sep_byte;
  }

  // --- Empty result → "." ---
  if buf.is_empty() {
    if preserve_trailing {
      let mut current_directory = PathBuf::from(".");
      current_directory.push("");
      return Cow::Owned(current_directory);
    }
    return Cow::Borrowed(Path::new("."));
  }

  // --- Prefix-only: preserve its component semantics or use its canonical suffix ---
  #[cfg(target_family = "windows")]
  if buf.len() == prefix_len
    && prefix_len > 0
    && let Some(suffix) = prefix_only_suffix
  {
    buf.push(suffix);
  }

  if preserve_trailing && buf.last() != Some(&sep_byte) {
    buf.push(sep_byte);
  }

  // SAFETY: `buf` was built entirely from:
  // - encoded bytes of OsStr components (valid platform encoding)
  // - ASCII separator bytes and ASCII '.' characters
  // This preserves the encoding invariants required by OsString.
  Cow::Owned(PathBuf::from(unsafe { OsString::from_encoded_bytes_unchecked(buf) }))
}

impl SugarPath for str {
  fn normalize(&self) -> Cow<'_, Path> {
    Path::new(self).normalize()
  }

  fn absolutize(&self) -> Cow<'_, Path> {
    Path::new(self).absolutize()
  }

  fn try_absolutize(&self) -> io::Result<Cow<'_, Path>> {
    Path::new(self).try_absolutize()
  }

  fn absolutize_with(&self, cwd: impl AsRef<Path> + Into<PathBuf>) -> Cow<'_, Path> {
    Path::new(self).absolutize_with(cwd)
  }

  fn relative(&self, base: impl AsRef<Path>) -> Cow<'_, Path> {
    Path::new(self).relative(base)
  }

  fn try_relative(&self, base: impl AsRef<Path>) -> io::Result<Cow<'_, Path>> {
    Path::new(self).try_relative(base)
  }

  fn relative_with(
    &self,
    base: impl AsRef<Path>,
    cwd: impl AsRef<Path> + Into<PathBuf>,
  ) -> Cow<'_, Path> {
    Path::new(self).relative_with(base, cwd)
  }

  fn to_slash(&self) -> Cow<'_, str> {
    if std::path::MAIN_SEPARATOR == '/' {
      Cow::Borrowed(self)
    } else {
      match replace_main_separator(self) {
        Some(replaced) => Cow::Owned(replaced),
        None => Cow::Borrowed(self),
      }
    }
  }

  fn try_to_slash(&self) -> Option<Cow<'_, str>> {
    Some(self.to_slash())
  }

  fn to_slash_lossy(&self) -> Cow<'_, str> {
    self.to_slash()
  }

  fn as_path(&self) -> &Path {
    Path::new(self)
  }
}

#[cfg(target_family = "windows")]
fn windows_paths_have_different_prefixes(base: &Path, target: &Path) -> bool {
  match (base.components().next(), target.components().next()) {
    (Some(Component::Prefix(base)), Some(Component::Prefix(target))) => {
      !windows_prefixes_eq_ignore_ascii_case(base.kind(), target.kind())
    }
    (Some(Component::Prefix(_)), _) | (_, Some(Component::Prefix(_))) => true,
    _ => false,
  }
}

#[cfg(target_family = "windows")]
fn windows_components_eq_ignore_ascii_case(from: &Component<'_>, to: &Component<'_>) -> bool {
  match (from, to) {
    (Component::Normal(from), Component::Normal(to)) => {
      from.as_encoded_bytes().eq_ignore_ascii_case(to.as_encoded_bytes())
    }
    (Component::Prefix(from), Component::Prefix(to)) => {
      windows_prefixes_eq_ignore_ascii_case(from.kind(), to.kind())
    }
    _ => from == to,
  }
}

#[cfg(target_family = "windows")]
fn windows_prefixes_eq_ignore_ascii_case(
  from: std::path::Prefix<'_>,
  to: std::path::Prefix<'_>,
) -> bool {
  use std::path::Prefix;

  let os_eq_ignore_ascii_case = |from: &std::ffi::OsStr, to: &std::ffi::OsStr| {
    from.as_encoded_bytes().eq_ignore_ascii_case(to.as_encoded_bytes())
  };

  match (from, to) {
    (Prefix::Disk(from), Prefix::Disk(to))
    | (Prefix::VerbatimDisk(from), Prefix::VerbatimDisk(to)) => from.eq_ignore_ascii_case(&to),
    (Prefix::UNC(from_server, from_share), Prefix::UNC(to_server, to_share))
    | (Prefix::VerbatimUNC(from_server, from_share), Prefix::VerbatimUNC(to_server, to_share)) => {
      os_eq_ignore_ascii_case(from_server, to_server)
        && os_eq_ignore_ascii_case(from_share, to_share)
    }
    (Prefix::DeviceNS(from), Prefix::DeviceNS(to))
    | (Prefix::Verbatim(from), Prefix::Verbatim(to)) => os_eq_ignore_ascii_case(from, to),
    _ => false,
  }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64", target_feature = "neon"))]
fn relative_str<'a>(target: &'a str, base: &str) -> Cow<'a, str> {
  let target = target.trim_end_matches('/');
  let base = base.trim_end_matches('/');
  relative_str_suffix_validated(target, base)
}

#[cfg(all(
  any(target_os = "macos", target_os = "linux"),
  any(test, all(target_os = "macos", target_arch = "aarch64", target_feature = "neon"))
))]
fn relative_str_suffix_validated<'a>(target: &'a str, base: &str) -> Cow<'a, str> {
  let common_byte_len = common_prefix_len_case_sensitive(target.as_bytes(), base.as_bytes());
  let at_boundary = (common_byte_len == target.len() && common_byte_len == base.len())
    || (common_byte_len == target.len() && base.as_bytes().get(common_byte_len) == Some(&b'/'))
    || (common_byte_len == base.len() && target.as_bytes().get(common_byte_len) == Some(&b'/'));
  let common_prefix = if at_boundary {
    common_byte_len
  } else {
    memrchr(b'/', &target.as_bytes()[..common_byte_len]).unwrap_or(0)
  };

  if at_boundary && common_byte_len == base.len() {
    // `base` is an exact component-boundary prefix of `target`, so every base
    // component and separator has already been validated when `target` is clean.
    if needs_relative_normalization(target) {
      return Cow::Owned(relative_str_slow(target, base));
    }
    return Cow::Borrowed(target[common_prefix..].trim_start_matches('/'));
  }

  let base_remaining = &base.as_bytes()[common_prefix..];
  let mut ups = 0u32;
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

  // Upward results are always owned. Shared dirty components normalize to the
  // same prefix on both sides, so only the unmatched suffixes can change the
  // result. Non-prefix zero-up cases still scan both full inputs.
  let needs_normalization = if ups == 0 {
    needs_relative_normalization(target) || needs_relative_normalization(base)
  } else {
    needs_relative_normalization(&target[common_prefix..])
      || needs_relative_normalization(&base[common_prefix..])
  };
  if needs_normalization {
    Cow::Owned(relative_str_slow(target, base))
  } else {
    relative_str_from_parts(target, common_prefix, ups as usize)
  }
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64", target_feature = "neon")))]
fn relative_str<'a>(target: &'a str, base: &str) -> Cow<'a, str> {
  let target = target.trim_end_matches('/');
  let base = base.trim_end_matches('/');
  if needs_relative_normalization(target) || needs_relative_normalization(base) {
    Cow::Owned(relative_str_slow(target, base))
  } else {
    relative_str_fast(target, base)
  }
}

#[cfg(not(target_family = "windows"))]
#[inline]
fn common_prefix_len_case_sensitive(left: &[u8], right: &[u8]) -> usize {
  #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
  {
    common_prefix_len_neon(left, right)
  }
  #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
  {
    common_prefix_len_scalar(left, right)
  }
}

#[cfg(not(target_family = "windows"))]
#[inline]
fn common_prefix_len_scalar(left: &[u8], right: &[u8]) -> usize {
  left
    .iter()
    .zip(right)
    .position(|(left, right)| left != right)
    .unwrap_or(left.len().min(right.len()))
}

#[cfg(all(not(target_family = "windows"), target_arch = "aarch64", target_feature = "neon"))]
#[inline]
fn common_prefix_len_neon(left: &[u8], right: &[u8]) -> usize {
  use std::arch::aarch64::{vceqq_u8, vld1q_u8, vminvq_u8, vst1q_u8};

  let len = left.len().min(right.len());
  let vector_end = len & !15;
  let mut offset = 0;

  while offset < vector_end {
    // SAFETY: `vector_end` is `len` rounded down to a multiple of 16, and
    // `offset` advances by exactly 16. Both loads therefore stay within the
    // shorter input, including when either slice starts at an unaligned address.
    let equal = unsafe {
      let left_chunk = vld1q_u8(left.as_ptr().add(offset));
      let right_chunk = vld1q_u8(right.as_ptr().add(offset));
      vceqq_u8(left_chunk, right_chunk)
    };
    // Equality lanes are all ones for a match and all zeroes for a mismatch.
    if unsafe { vminvq_u8(equal) } != u8::MAX {
      let mut equal_bytes = [0; 16];
      // SAFETY: `equal_bytes` has exactly enough initialized space for one
      // 16-byte vector store.
      unsafe { vst1q_u8(equal_bytes.as_mut_ptr(), equal) };
      let mismatch_bits = !u128::from_le_bytes(equal_bytes);
      debug_assert_ne!(mismatch_bits, 0);
      return offset + mismatch_bits.trailing_zeros() as usize / 8;
    }
    offset += 16;
  }

  offset + common_prefix_len_scalar(&left[offset..len], &right[offset..len])
}

/// Check if a path contains components or separators that need normalization.
/// Uses `memchr` to jump between `/` positions — most bytes in a path aren't `/`,
/// so this skips the vast majority of the input.
#[inline]
fn needs_relative_normalization(path: &str) -> bool {
  let bytes = path.as_bytes();
  if bytes.len() > 1 && bytes.last() == Some(&b'/') {
    return true;
  }
  if bytes.first() == Some(&b'.') {
    if bytes.len() == 1 || bytes.get(1) == Some(&b'/') {
      return true;
    }
    if bytes.get(1) == Some(&b'.') && (bytes.len() == 2 || bytes.get(2) == Some(&b'/')) {
      return true;
    }
  }
  let mut offset = 0;
  while let Some(pos) = memchr(b'/', &bytes[offset..]) {
    let slash = offset + pos;
    if slash + 1 < bytes.len() && bytes[slash + 1] == b'/' {
      return true;
    }
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
#[cfg(not(all(target_os = "macos", target_arch = "aarch64", target_feature = "neon")))]
fn relative_str_fast<'a>(target: &'a str, base: &str) -> Cow<'a, str> {
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
      common_prefix_len_case_sensitive(target.as_bytes(), base.as_bytes())
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
  if ups == 0 {
    return Cow::Borrowed(target_suffix);
  }
  let suffix_iter = if target_suffix.is_empty() { None } else { Some(target_suffix) };
  let mut result = String::with_capacity(ups * 3 + target_suffix.len());
  std::iter::repeat_n("..", ups).chain(suffix_iter).for_each(|s| {
    if !result.is_empty() {
      result.push('/');
    }
    result.push_str(s);
  });
  Cow::Owned(result)
}

#[cfg(all(
  any(target_os = "macos", target_os = "linux"),
  any(test, all(target_os = "macos", target_arch = "aarch64", target_feature = "neon"))
))]
fn relative_str_from_parts<'a>(target: &'a str, common_prefix: usize, ups: usize) -> Cow<'a, str> {
  let target_suffix = target[common_prefix..].trim_start_matches('/');
  if ups == 0 {
    return Cow::Borrowed(target_suffix);
  }
  let suffix_iter = if target_suffix.is_empty() { None } else { Some(target_suffix) };
  let mut result = String::with_capacity(ups * 3 + target_suffix.len());
  std::iter::repeat_n("..", ups).chain(suffix_iter).for_each(|s| {
    if !result.is_empty() {
      result.push('/');
    }
    result.push_str(s);
  });
  Cow::Owned(result)
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

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod relative_str_tests {
  use std::borrow::Cow;

  use super::{
    needs_relative_normalization, relative_str, relative_str_slow, relative_str_suffix_validated,
  };

  fn assert_dispatch_and_suffix_validation_match_full_normalization(target: &str, base: &str) {
    let dispatched = relative_str(target, base);
    let target = target.trim_end_matches('/');
    let base = base.trim_end_matches('/');
    let suffix_validated = relative_str_suffix_validated(target, base);
    let expected = relative_str_slow(target, base);
    assert_eq!(dispatched, expected, "production dispatch: target {target:?}, base {base:?}");
    assert_eq!(suffix_validated, expected, "suffix validation: target {target:?}, base {base:?}");

    let dirty = needs_relative_normalization(target) || needs_relative_normalization(base);
    let base_is_component_prefix =
      target.strip_prefix(base).is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('/'));
    let should_borrow = !dirty && base_is_component_prefix;
    assert_eq!(
      matches!(dispatched, Cow::Borrowed(_)),
      should_borrow,
      "target {target:?}, base {base:?} returned the wrong Cow variant through production dispatch",
    );
    assert_eq!(
      matches!(suffix_validated, Cow::Borrowed(_)),
      should_borrow,
      "target {target:?}, base {base:?} returned the wrong Cow variant through suffix validation",
    );
  }

  fn short_absolute_spellings() -> Vec<String> {
    let components = ["", ".", "..", "a", "A", "b"];
    let mut paths = Vec::new();
    for depth in 0..=3u32 {
      for mut index in 0..components.len().pow(depth) {
        let mut path = String::from("/");
        for component_index in 0..depth {
          if component_index > 0 {
            path.push('/');
          }
          path.push_str(components[index % components.len()]);
          index /= components.len();
        }
        paths.push(path.clone());
        if path.len() > 1 {
          path.push('/');
          paths.push(path);
        }
      }
    }
    paths.sort_unstable();
    paths.dedup();
    paths
  }

  #[test]
  fn production_dispatch_and_suffix_validation_match_short_path_oracle() {
    let paths = short_absolute_spellings();
    assert_eq!(paths.len(), 474, "bounded short-path corpus changed");
    for target in &paths {
      for base in &paths {
        assert_dispatch_and_suffix_validation_match_full_normalization(target, base);
      }
    }
  }

  #[test]
  fn production_dispatch_and_suffix_validation_handle_multibyte_paths() {
    let paths = ["/é", "/ê", "/é/a", "/ê/a", "/猫", "/猫/src", "/猫/../src/a", "/猫/../src/b"];
    for target in paths {
      for base in paths {
        assert_dispatch_and_suffix_validation_match_full_normalization(target, base);
      }
    }
  }
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

#[cfg(all(test, not(target_family = "windows"), target_arch = "aarch64", target_feature = "neon"))]
mod common_prefix_tests {
  use super::{common_prefix_len_neon, common_prefix_len_scalar};

  fn bytes(len: usize) -> Vec<u8> {
    (0..len).map(|index| ((index * 37 + 11) % 251) as u8).collect()
  }

  #[test]
  fn neon_matches_scalar_for_lengths_and_every_mismatch() {
    for len in 0..=256 {
      let left = bytes(len);
      assert_eq!(common_prefix_len_neon(&left, &left), len, "equal input length {len}");

      for mismatch in 0..len {
        let mut right = left.clone();
        right[mismatch] ^= u8::MAX;
        assert_eq!(
          common_prefix_len_neon(&left, &right),
          common_prefix_len_scalar(&left, &right),
          "input length {len}, mismatch {mismatch}",
        );
      }
    }
  }

  #[test]
  fn neon_matches_scalar_for_every_length_pair() {
    let input = bytes(256);
    for left_len in 0..=256 {
      for right_len in 0..=256 {
        assert_eq!(
          common_prefix_len_neon(&input[..left_len], &input[..right_len]),
          common_prefix_len_scalar(&input[..left_len], &input[..right_len]),
          "left length {left_len}, right length {right_len}",
        );
      }
    }
  }

  #[test]
  fn neon_handles_all_sixteen_byte_alignment_pairs() {
    for left_offset in 0..16 {
      for right_offset in 0..16 {
        for len in 0..=256 {
          let mut left = [0xa5; 288];
          let mut right = [0x5a; 288];
          for index in 0..len {
            let value = ((index * 37 + 11) % 251) as u8;
            left[left_offset + index] = value;
            right[right_offset + index] = value;
          }
          let left = &left[left_offset..left_offset + len];
          let right = &right[right_offset..right_offset + len];
          assert_eq!(
            common_prefix_len_neon(left, right),
            len,
            "left offset {left_offset}, right offset {right_offset}, length {len}",
          );
        }
      }
    }
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
mod normalization_classifier_tests {
  use super::*;

  fn normalize_without_classifier(path: &Path, trailing: TrailingSeparator) -> Cow<'_, Path> {
    normalize_inner(
      path.components().peekable(),
      path.as_os_str().len(),
      trailing == TrailingSeparator::Preserve && has_trailing_separator(path),
      {
        #[cfg(target_family = "windows")]
        {
          windows_drive_spelling(path)
        }
        #[cfg(not(target_family = "windows"))]
        {
          None
        }
      },
    )
  }

  fn assert_classifier_matches_full_normalizer(path: &Path) {
    for trailing in [TrailingSeparator::Preserve, TrailingSeparator::Strip] {
      let classified = normalize_path(path, trailing);
      let rebuilt = normalize_without_classifier(path, trailing);
      assert_eq!(
        classified.as_os_str(),
        rebuilt.as_os_str(),
        "classifier skipped required work for {path:?}",
      );
    }
  }

  #[cfg(unix)]
  #[test]
  fn unix_classifier_matches_full_normalizer_for_short_arbitrary_bytes() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    const ALPHABET: &[u8] = &[b'a', b'.', b'/', 0x80, 0xff];
    let mut input_count = 0;
    for len in 0..=6 {
      for mut ordinal in 0..ALPHABET.len().pow(len as u32) {
        let mut bytes = vec![0; len];
        for byte in &mut bytes {
          *byte = ALPHABET[ordinal % ALPHABET.len()];
          ordinal /= ALPHABET.len();
        }
        assert_classifier_matches_full_normalizer(Path::new(&OsString::from_vec(bytes)));
        input_count += 1;
      }
    }
    assert_eq!(input_count, 19_531, "bounded Unix classifier corpus changed");
  }

  #[cfg(windows)]
  #[test]
  fn windows_classifier_matches_full_normalizer_for_short_arbitrary_wide_units() {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};

    const ALPHABET: &[u16] = &[b'a' as u16, b'.' as u16, b'\\' as u16, b'/' as u16, 0xd800];
    let mut input_count = 0;
    for len in 0..=6 {
      for mut ordinal in 0..ALPHABET.len().pow(len as u32) {
        let mut units = vec![0; len];
        for unit in &mut units {
          *unit = ALPHABET[ordinal % ALPHABET.len()];
          ordinal /= ALPHABET.len();
        }
        assert_classifier_matches_full_normalizer(Path::new(&OsString::from_wide(&units)));
        input_count += 1;
      }
    }
    assert_eq!(input_count, 19_531, "bounded Windows classifier corpus changed");
  }
}
