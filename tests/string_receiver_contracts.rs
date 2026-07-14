#![cfg(any(target_family = "unix", target_family = "windows"))]

use std::{borrow::Cow, path::Path};

use sugar_path::SugarPath;

fn assert_borrowed_from_receiver(
  receiver: &Path,
  result: Cow<'_, Path>,
  expected: &Path,
  context: &str,
) {
  assert_eq!(result.as_os_str(), expected.as_os_str(), "{context}");
  let Cow::Borrowed(result) = result else {
    panic!("{context}: expected a borrowed result");
  };

  let receiver = receiver.as_os_str().as_encoded_bytes();
  let result = result.as_os_str().as_encoded_bytes();
  let receiver_start = receiver.as_ptr() as usize;
  let receiver_end = receiver_start + receiver.len();
  let result_start = result.as_ptr() as usize;
  let result_end = result_start + result.len();
  assert!(
    result_start >= receiver_start && result_end <= receiver_end,
    "{context}: result did not borrow from the receiver",
  );
}

fn assert_receiver_contracts(input: &str, base: &str, expected_relative: &str, label: &str) {
  let receiver = Path::new(input);
  assert_borrowed_from_receiver(
    receiver,
    input.normalize(),
    receiver,
    &format!("{label} str normalize"),
  );
  assert_borrowed_from_receiver(
    receiver,
    input.absolutize(),
    receiver,
    &format!("{label} str absolutize"),
  );
  assert_borrowed_from_receiver(
    receiver,
    input.try_absolutize().expect("clean absolute str should resolve"),
    receiver,
    &format!("{label} str try_absolutize"),
  );
  assert_borrowed_from_receiver(
    receiver,
    input.absolutize_with("relative/unused"),
    receiver,
    &format!("{label} str absolutize_with"),
  );
  assert_borrowed_from_receiver(
    receiver,
    input.relative(base),
    Path::new(expected_relative),
    &format!("{label} str relative"),
  );
  assert_borrowed_from_receiver(
    receiver,
    input.try_relative(base).expect("absolute str paths should resolve"),
    Path::new(expected_relative),
    &format!("{label} str try_relative"),
  );
  assert_borrowed_from_receiver(
    receiver,
    input.relative_with(base, "relative/unused"),
    Path::new(expected_relative),
    &format!("{label} str relative_with"),
  );

  let owned = String::from(input);
  let receiver = Path::new(&owned);
  assert_borrowed_from_receiver(
    receiver,
    owned.normalize(),
    receiver,
    &format!("{label} String normalize"),
  );
  assert_borrowed_from_receiver(
    receiver,
    owned.absolutize(),
    receiver,
    &format!("{label} String absolutize"),
  );
  assert_borrowed_from_receiver(
    receiver,
    owned.try_absolutize().expect("clean absolute String should resolve"),
    receiver,
    &format!("{label} String try_absolutize"),
  );
  assert_borrowed_from_receiver(
    receiver,
    owned.absolutize_with("relative/unused"),
    receiver,
    &format!("{label} String absolutize_with"),
  );
  assert_borrowed_from_receiver(
    receiver,
    owned.relative(base),
    Path::new(expected_relative),
    &format!("{label} String relative"),
  );
  assert_borrowed_from_receiver(
    receiver,
    owned.try_relative(base).expect("absolute String paths should resolve"),
    Path::new(expected_relative),
    &format!("{label} String try_relative"),
  );
  assert_borrowed_from_receiver(
    receiver,
    owned.relative_with(base, "relative/unused"),
    Path::new(expected_relative),
    &format!("{label} String relative_with"),
  );
}

#[test]
fn utf8_string_receivers_borrow_only_from_the_receiver() {
  #[cfg(target_family = "unix")]
  assert_receiver_contracts("/workspace/β/src/lib.rs", "/workspace/β", "src/lib.rs", "Unix");

  #[cfg(target_family = "windows")]
  assert_receiver_contracts(
    r"C:\workspace\β\src\lib.rs",
    r"c:\workspace\β",
    r"src\lib.rs",
    "Windows",
  );
}
