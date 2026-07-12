use std::{
  ffi::{OsStr, OsString},
  marker::PhantomData,
};

// Invariance prevents a fragment created by one arena session from being used
// with another arena, even when both sessions are nested in the same function.
type Brand<'id> = PhantomData<fn(&'id ()) -> &'id ()>;

#[derive(Clone, Copy)]
pub(crate) struct EncodedFragment<'id> {
  start: u16,
  end: u16,
  brand: Brand<'id>,
}

pub(crate) struct EncodedArena<'id, const N: usize> {
  bytes: [u8; N],
  len: usize,
  brand: Brand<'id>,
}

pub(crate) fn with_encoded_arena<const N: usize, R>(
  f: impl for<'id> FnOnce(EncodedArena<'id, N>) -> R,
) -> R {
  f(EncodedArena { bytes: [0; N], len: 0, brand: PhantomData })
}

impl<'id, const N: usize> EncodedArena<'id, N> {
  pub(crate) fn store_os_str(&mut self, value: &OsStr) -> Option<EncodedFragment<'id>> {
    let source = value.as_encoded_bytes();
    let start = self.len;
    let end = start.checked_add(source.len())?;
    if end > N || end > usize::from(u16::MAX) {
      return None;
    }

    self.bytes[start..end].copy_from_slice(source);
    self.len = end;
    Some(EncodedFragment { start: start as u16, end: end as u16, brand: PhantomData })
  }

  pub(crate) fn push_to(&self, output: &mut OsString, fragment: EncodedFragment<'id>) {
    let bytes = &self.bytes[usize::from(fragment.start)..usize::from(fragment.end)];
    // SAFETY: `EncodedFragment` can only be created by `store_os_str` in this
    // branded, append-only arena. Its range therefore contains one complete
    // `OsStr::as_encoded_bytes()` result from this Rust version and target.
    #[expect(unsafe_code, reason = "single audited native-encoding reconstruction")]
    let value = unsafe { OsStr::from_encoded_bytes_unchecked(bytes) };
    output.push(value);
  }

  #[cfg(target_family = "windows")]
  pub(crate) fn is_standalone_windows_relative(&self, fragment: EncodedFragment<'id>) -> bool {
    let bytes = &self.bytes[usize::from(fragment.start)..usize::from(fragment.end)];
    !matches!(bytes.first(), Some(b'/' | b'\\'))
      && !(bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
  }
}
