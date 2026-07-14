#![cfg(target_family = "windows")]

use std::{
  borrow::Cow,
  ffi::{OsStr, OsString},
  os::windows::ffi::{OsStrExt, OsStringExt},
  path::{Component, Path, PathBuf, Prefix},
};

use sugar_path::SugarPath;

const FIRST_INVALID_UNIT: u16 = 0xd800;
const SECOND_INVALID_UNIT: u16 = 0xd801;

#[derive(Clone, Copy, Debug)]
enum RootSlot {
  UncServer,
  UncShare,
  VerbatimUncServer,
  VerbatimUncShare,
  Device,
  Verbatim,
}

#[derive(Clone, Copy)]
struct RootCase {
  name: &'static str,
  slot: RootSlot,
  root_before_unit: &'static str,
  root_after_unit: &'static str,
  folded_before_unit: &'static str,
  folded_after_unit: &'static str,
  lowercase_unicode_root: &'static str,
  uppercase_unicode_root: &'static str,
}

const ROOT_CASES: &[RootCase] = &[
  RootCase {
    name: "UNC server",
    slot: RootSlot::UncServer,
    root_before_unit: r"\\SeRvEr-",
    root_after_unit: r"\ShArE",
    folded_before_unit: r"\\sErVeR-",
    folded_after_unit: r"\sHaRe",
    lowercase_unicode_root: r"\\ä\share",
    uppercase_unicode_root: r"\\Ä\share",
  },
  RootCase {
    name: "UNC share",
    slot: RootSlot::UncShare,
    root_before_unit: r"\\SeRvEr\ShArE-",
    root_after_unit: "",
    folded_before_unit: r"\\sErVeR\sHaRe-",
    folded_after_unit: "",
    lowercase_unicode_root: r"\\server\ä",
    uppercase_unicode_root: r"\\server\Ä",
  },
  RootCase {
    name: "verbatim UNC server",
    slot: RootSlot::VerbatimUncServer,
    root_before_unit: r"\\?\UNC\SeRvEr-",
    root_after_unit: r"\ShArE",
    folded_before_unit: r"\\?\UNC\sErVeR-",
    folded_after_unit: r"\sHaRe",
    lowercase_unicode_root: r"\\?\UNC\ä\share",
    uppercase_unicode_root: r"\\?\UNC\Ä\share",
  },
  RootCase {
    name: "verbatim UNC share",
    slot: RootSlot::VerbatimUncShare,
    root_before_unit: r"\\?\UNC\SeRvEr\ShArE-",
    root_after_unit: "",
    folded_before_unit: r"\\?\UNC\sErVeR\sHaRe-",
    folded_after_unit: "",
    lowercase_unicode_root: r"\\?\UNC\server\ä",
    uppercase_unicode_root: r"\\?\UNC\server\Ä",
  },
  RootCase {
    name: "device namespace",
    slot: RootSlot::Device,
    root_before_unit: r"\\.\PiPe-",
    root_after_unit: "",
    folded_before_unit: r"\\.\pIpE-",
    folded_after_unit: "",
    lowercase_unicode_root: r"\\.\ä",
    uppercase_unicode_root: r"\\.\Ä",
  },
  RootCase {
    name: "generic verbatim",
    slot: RootSlot::Verbatim,
    root_before_unit: r"\\?\GlObAl-",
    root_after_unit: "",
    folded_before_unit: r"\\?\gLoBaL-",
    folded_after_unit: "",
    lowercase_unicode_root: r"\\?\ä",
    uppercase_unicode_root: r"\\?\Ä",
  },
];

fn path_with_unit(before: &str, unit: u16, after: &str, tail: &str) -> PathBuf {
  let mut wide: Vec<u16> = before.encode_utf16().collect();
  wide.push(unit);
  wide.extend(after.encode_utf16());
  wide.extend(tail.encode_utf16());
  PathBuf::from(OsString::from_wide(&wide))
}

fn path_with_tail(root: &str, tail: &str) -> PathBuf {
  let mut wide: Vec<u16> = root.encode_utf16().collect();
  wide.extend(tail.encode_utf16());
  PathBuf::from(OsString::from_wide(&wide))
}

fn wide(path: &Path) -> Vec<u16> {
  path.as_os_str().encode_wide().collect()
}

fn assert_wide_eq(actual: &Path, expected: &Path, context: &str) {
  assert_eq!(wide(actual), wide(expected), "{context}");
}

fn root_identifier(path: &Path, slot: RootSlot) -> &OsStr {
  let Some(Component::Prefix(prefix)) = path.components().next() else {
    panic!("expected {slot:?} prefix, found {path:?}");
  };

  match (slot, prefix.kind()) {
    (RootSlot::UncServer, Prefix::UNC(server, _)) => server,
    (RootSlot::UncShare, Prefix::UNC(_, share)) => share,
    (RootSlot::VerbatimUncServer, Prefix::VerbatimUNC(server, _)) => server,
    (RootSlot::VerbatimUncShare, Prefix::VerbatimUNC(_, share)) => share,
    (RootSlot::Device, Prefix::DeviceNS(device)) => device,
    (RootSlot::Verbatim, Prefix::Verbatim(name)) => name,
    (_, actual) => panic!("expected {slot:?} prefix, found {actual:?} in {path:?}"),
  }
}

fn assert_slot_contains_unit(path: &Path, slot: RootSlot, unit: u16) {
  assert!(
    root_identifier(path, slot).encode_wide().any(|actual| actual == unit),
    "expected {slot:?} identifier to contain {unit:#06x}: {path:?}",
  );
}

fn assert_file_result(result: Cow<'_, Path>, context: &str) {
  assert_wide_eq(&result, Path::new("file"), context);
}

fn assert_owned_target(result: Cow<'_, Path>, target: &Path, context: &str) {
  let Cow::Owned(result) = result else {
    panic!("{context}: expected an owned normalized target");
  };
  assert_wide_eq(&result, target, context);
}

fn assert_same_encoded_root(case: RootCase) {
  let target =
    path_with_unit(case.root_before_unit, FIRST_INVALID_UNIT, case.root_after_unit, r"\BASE\file");
  let base =
    path_with_unit(case.folded_before_unit, FIRST_INVALID_UNIT, case.folded_after_unit, r"\base");
  assert_slot_contains_unit(&target, case.slot, FIRST_INVALID_UNIT);
  assert_slot_contains_unit(&base, case.slot, FIRST_INVALID_UNIT);

  assert_file_result(target.relative(&base), &format!("{} relative", case.name));
  assert_file_result(
    target.try_relative(&base).expect("absolute inputs do not need cwd"),
    &format!("{} try_relative", case.name),
  );
  assert_file_result(
    target.relative_with(&base, "not/absolute"),
    &format!("{} relative_with", case.name),
  );
}

fn assert_distinct_encoded_root(case: RootCase) {
  let target_root =
    path_with_unit(case.root_before_unit, FIRST_INVALID_UNIT, case.root_after_unit, "");
  let base_root =
    path_with_unit(case.root_before_unit, SECOND_INVALID_UNIT, case.root_after_unit, "");
  assert_eq!(
    target_root.to_string_lossy(),
    base_root.to_string_lossy(),
    "{}: the lossy oracle must be unable to distinguish the units",
    case.name,
  );

  let target =
    path_with_unit(case.root_before_unit, FIRST_INVALID_UNIT, case.root_after_unit, r"\BASE\file");
  let base =
    path_with_unit(case.root_before_unit, SECOND_INVALID_UNIT, case.root_after_unit, r"\base");
  assert_slot_contains_unit(&target, case.slot, FIRST_INVALID_UNIT);
  assert_slot_contains_unit(&base, case.slot, SECOND_INVALID_UNIT);

  assert_owned_target(target.relative(&base), &target, &format!("{} relative", case.name));
  assert_owned_target(
    target.try_relative(&base).expect("absolute inputs do not need cwd"),
    &target,
    &format!("{} try_relative", case.name),
  );
  assert_owned_target(
    target.relative_with(&base, "not/absolute"),
    &target,
    &format!("{} relative_with", case.name),
  );
}

fn assert_non_ascii_root_is_not_folded(case: RootCase) {
  let target = path_with_tail(case.lowercase_unicode_root, r"\BASE\file");
  let base = path_with_tail(case.uppercase_unicode_root, r"\base");
  let _ = root_identifier(&target, case.slot);
  let _ = root_identifier(&base, case.slot);

  assert_owned_target(target.relative(&base), &target, &format!("{} relative", case.name));
  assert_owned_target(
    target.try_relative(&base).expect("absolute inputs do not need cwd"),
    &target,
    &format!("{} try_relative", case.name),
  );
  assert_owned_target(
    target.relative_with(&base, "not/absolute"),
    &target,
    &format!("{} relative_with", case.name),
  );
}

#[test]
fn encoded_root_identifiers_compare_only_ascii_case_and_exact_native_units() {
  for &case in ROOT_CASES {
    assert_same_encoded_root(case);
    assert_distinct_encoded_root(case);
    assert_non_ascii_root_is_not_folded(case);
  }

  let target = path_with_unit(r"\\.\NaMe-", FIRST_INVALID_UNIT, "", r"\BASE\file");
  let base = path_with_unit(r"\\?\nAmE-", FIRST_INVALID_UNIT, "", r"\base");
  assert_slot_contains_unit(&target, RootSlot::Device, FIRST_INVALID_UNIT);
  assert_slot_contains_unit(&base, RootSlot::Verbatim, FIRST_INVALID_UNIT);
  assert_owned_target(target.relative(&base), &target, "device versus verbatim relative");
  assert_owned_target(
    target.try_relative(&base).expect("absolute inputs do not need cwd"),
    &target,
    "device versus verbatim try_relative",
  );
  assert_owned_target(
    target.relative_with(&base, "not/absolute"),
    &target,
    "device versus verbatim relative_with",
  );
}

fn assert_absolutizes_exactly(result: Cow<'_, Path>, expected: &Path, context: &str) {
  assert_wide_eq(&result, expected, context);
}

#[test]
fn absolutization_preserves_encoded_root_identifiers() {
  for &case in ROOT_CASES {
    let clean = path_with_unit(
      case.root_before_unit,
      FIRST_INVALID_UNIT,
      case.root_after_unit,
      r"\base\file",
    );
    assert_slot_contains_unit(&clean, case.slot, FIRST_INVALID_UNIT);
    assert_absolutizes_exactly(clean.absolutize(), &clean, &format!("{} clean strict", case.name));
    assert_absolutizes_exactly(
      clean.try_absolutize().expect("absolute input does not need cwd"),
      &clean,
      &format!("{} clean try", case.name),
    );
    assert_absolutizes_exactly(
      clean.absolutize_with("not/absolute"),
      &clean,
      &format!("{} clean explicit", case.name),
    );

    let dirty = path_with_unit(
      case.root_before_unit,
      FIRST_INVALID_UNIT,
      case.root_after_unit,
      r"\base\.\file\",
    );
    let normalized = path_with_unit(
      case.root_before_unit,
      FIRST_INVALID_UNIT,
      case.root_after_unit,
      r"\base\file",
    );
    let Cow::Owned(strict) = dirty.absolutize() else {
      panic!("{} dirty strict: expected owned result", case.name);
    };
    assert_wide_eq(&strict, &normalized, &format!("{} dirty strict", case.name));
    let Cow::Owned(fallible) = dirty.try_absolutize().expect("absolute input does not need cwd")
    else {
      panic!("{} dirty try: expected owned result", case.name);
    };
    assert_wide_eq(&fallible, &normalized, &format!("{} dirty try", case.name));
    let Cow::Owned(explicit) = dirty.absolutize_with("not/absolute") else {
      panic!("{} dirty explicit: expected owned result", case.name);
    };
    assert_wide_eq(&explicit, &normalized, &format!("{} dirty explicit", case.name));

    let cwd =
      path_with_unit(case.root_before_unit, FIRST_INVALID_UNIT, case.root_after_unit, r"\base");
    let input = Path::new(r"pkg\.\file");
    let expected = path_with_unit(
      case.root_before_unit,
      FIRST_INVALID_UNIT,
      case.root_after_unit,
      r"\base\pkg\file",
    );
    let Cow::Owned(borrowed_cwd) = input.absolutize_with(cwd.as_path()) else {
      panic!("{} borrowed cwd: expected owned result", case.name);
    };
    assert_wide_eq(&borrowed_cwd, &expected, &format!("{} borrowed cwd", case.name));
    let Cow::Owned(owned_cwd) = input.absolutize_with(cwd) else {
      panic!("{} owned cwd: expected owned result", case.name);
    };
    assert_wide_eq(&owned_cwd, &expected, &format!("{} owned cwd", case.name));

    let cwd =
      path_with_unit(case.root_before_unit, FIRST_INVALID_UNIT, case.root_after_unit, r"\base");
    let root_relative = Path::new(r"\pkg\.\file\");
    let expected =
      path_with_unit(case.root_before_unit, FIRST_INVALID_UNIT, case.root_after_unit, r"\pkg\file");
    let Cow::Owned(borrowed_cwd) = root_relative.absolutize_with(cwd.as_path()) else {
      panic!("{} root-relative borrowed cwd: expected owned result", case.name);
    };
    assert_wide_eq(&borrowed_cwd, &expected, &format!("{} root-relative borrowed cwd", case.name));
    let Cow::Owned(owned_cwd) = root_relative.absolutize_with(cwd) else {
      panic!("{} root-relative owned cwd: expected owned result", case.name);
    };
    assert_wide_eq(&owned_cwd, &expected, &format!("{} root-relative owned cwd", case.name));
  }
}
