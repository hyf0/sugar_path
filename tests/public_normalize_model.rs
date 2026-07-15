#![cfg(any(target_family = "unix", target_family = "windows"))]

use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use sugar_path::{SugarPath, SugarPathBuf};

const ATOMS: &[Atom] = &[
  Atom::Name("a"),
  Atom::Name("A"),
  Atom::Name("b"),
  Atom::Name("β"),
  #[cfg(target_family = "unix")]
  Atom::Name("literal\\"),
  Atom::Name(".config"),
  Atom::Name("..cache"),
  Atom::Current,
  Atom::Parent,
];

#[cfg(target_family = "unix")]
const EXPECTED_CASES: usize = 6_560;
#[cfg(target_family = "windows")]
const EXPECTED_CASES: usize = 14_040;

#[derive(Clone, Copy, Debug)]
enum Atom {
  Name(&'static str),
  Current,
  Parent,
}

impl Atom {
  fn spelling(self) -> &'static str {
    match self {
      Self::Name(name) => name,
      Self::Current => ".",
      Self::Parent => "..",
    }
  }
}

#[derive(Clone, Copy, Debug)]
enum InputRoot {
  Relative,
  #[cfg(target_family = "unix")]
  Unix,
  #[cfg(target_family = "windows")]
  RootRelative,
  #[cfg(target_family = "windows")]
  Disk(u8),
  #[cfg(target_family = "windows")]
  DriveRelative(u8),
}

impl InputRoot {
  fn is_rooted(self) -> bool {
    match self {
      Self::Relative => false,
      #[cfg(target_family = "unix")]
      Self::Unix => true,
      #[cfg(target_family = "windows")]
      Self::RootRelative | Self::Disk(_) => true,
      #[cfg(target_family = "windows")]
      Self::DriveRelative(_) => false,
    }
  }
}

#[derive(Clone, Debug)]
struct InputCase {
  root: InputRoot,
  atoms: Vec<Atom>,
}

#[derive(Clone, Copy, Debug)]
enum SeparatorSpelling {
  Native,
  RedundantNative,
  #[cfg(target_family = "windows")]
  Forward,
}

impl SeparatorSpelling {
  fn separator(self) -> &'static str {
    match self {
      Self::Native => native_separator(),
      Self::RedundantNative => redundant_separator(),
      #[cfg(target_family = "windows")]
      Self::Forward => "/",
    }
  }
}

#[derive(Clone, Copy, Debug)]
struct Mode {
  name: &'static str,
  separators: SeparatorSpelling,
  trailing: bool,
}

#[cfg(target_family = "unix")]
const MODES: &[Mode] = &[
  Mode { name: "native", separators: SeparatorSpelling::Native, trailing: false },
  Mode { name: "native_trailing", separators: SeparatorSpelling::Native, trailing: true },
  Mode { name: "redundant", separators: SeparatorSpelling::RedundantNative, trailing: false },
  Mode {
    name: "redundant_trailing",
    separators: SeparatorSpelling::RedundantNative,
    trailing: true,
  },
];

#[cfg(target_family = "windows")]
const MODES: &[Mode] = &[
  Mode { name: "native", separators: SeparatorSpelling::Native, trailing: false },
  Mode { name: "native_trailing", separators: SeparatorSpelling::Native, trailing: true },
  Mode { name: "redundant", separators: SeparatorSpelling::RedundantNative, trailing: false },
  Mode {
    name: "redundant_trailing",
    separators: SeparatorSpelling::RedundantNative,
    trailing: true,
  },
  Mode { name: "forward", separators: SeparatorSpelling::Forward, trailing: false },
  Mode { name: "forward_trailing", separators: SeparatorSpelling::Forward, trailing: true },
];

#[derive(Clone, Copy, Debug)]
enum NormalizedAtom {
  Name(&'static str),
  Parent,
}

#[cfg(target_family = "unix")]
fn platform_name() -> &'static str {
  "unix"
}

#[cfg(target_family = "windows")]
fn platform_name() -> &'static str {
  "windows"
}

#[cfg(target_family = "unix")]
fn native_separator() -> &'static str {
  "/"
}

#[cfg(target_family = "windows")]
fn native_separator() -> &'static str {
  r"\"
}

#[cfg(target_family = "unix")]
fn redundant_separator() -> &'static str {
  "//"
}

#[cfg(target_family = "windows")]
fn redundant_separator() -> &'static str {
  r"\\"
}

#[cfg(target_family = "unix")]
fn root_separator(spelling: SeparatorSpelling) -> &'static str {
  spelling.separator()
}

#[cfg(target_family = "windows")]
fn root_separator(spelling: SeparatorSpelling) -> &'static str {
  match spelling {
    SeparatorSpelling::Forward => "/",
    SeparatorSpelling::Native | SeparatorSpelling::RedundantNative => r"\",
  }
}

fn roots() -> &'static [InputRoot] {
  #[cfg(target_family = "unix")]
  {
    &[InputRoot::Relative, InputRoot::Unix]
  }
  #[cfg(target_family = "windows")]
  {
    &[
      InputRoot::Relative,
      InputRoot::RootRelative,
      InputRoot::Disk(b'c'),
      InputRoot::DriveRelative(b'c'),
    ]
  }
}

fn generated_atom_sequences() -> Vec<Vec<Atom>> {
  let mut sequences = Vec::new();
  for depth in 0..=3 {
    for mut ordinal in 0..ATOMS.len().pow(depth) {
      let mut sequence = Vec::with_capacity(depth as usize);
      for _ in 0..depth {
        sequence.push(ATOMS[ordinal % ATOMS.len()]);
        ordinal /= ATOMS.len();
      }
      sequences.push(sequence);
    }
  }
  sequences
}

fn generated_cases() -> Vec<InputCase> {
  generated_atom_sequences()
    .into_iter()
    .flat_map(|atoms| {
      roots().iter().map(move |root| InputCase { root: *root, atoms: atoms.clone() })
    })
    .collect()
}

fn render_input(case: &InputCase, mode: Mode) -> String {
  let separator = mode.separators.separator();
  let mut rendered = match case.root {
    InputRoot::Relative => String::new(),
    #[cfg(target_family = "unix")]
    InputRoot::Unix => String::from(root_separator(mode.separators)),
    #[cfg(target_family = "windows")]
    InputRoot::RootRelative => String::from(root_separator(mode.separators)),
    #[cfg(target_family = "windows")]
    InputRoot::Disk(drive) => {
      format!("{}:{}", char::from(drive), root_separator(mode.separators))
    }
    #[cfg(target_family = "windows")]
    InputRoot::DriveRelative(drive) => format!("{}:", char::from(drive)),
  };

  for (index, atom) in case.atoms.iter().enumerate() {
    if index > 0 {
      rendered.push_str(separator);
    }
    rendered.push_str(atom.spelling());
  }
  if mode.trailing && !case.atoms.is_empty() {
    rendered.push_str(separator);
  }
  rendered
}

fn normalized_atoms(case: &InputCase) -> Vec<NormalizedAtom> {
  let mut normalized = Vec::new();
  for atom in &case.atoms {
    match *atom {
      Atom::Name(name) => normalized.push(NormalizedAtom::Name(name)),
      Atom::Current => {}
      Atom::Parent => match normalized.last() {
        Some(NormalizedAtom::Name(_)) => {
          normalized.pop();
        }
        Some(NormalizedAtom::Parent) | None if !case.root.is_rooted() => {
          normalized.push(NormalizedAtom::Parent);
        }
        Some(NormalizedAtom::Parent) | None => {}
      },
    }
  }
  normalized
}

fn expected_normalized(case: &InputCase, input: &str) -> PathBuf {
  let components = normalized_atoms(case);
  let mut expected = match case.root {
    InputRoot::Relative => String::new(),
    #[cfg(target_family = "unix")]
    InputRoot::Unix => String::from("/"),
    #[cfg(target_family = "windows")]
    InputRoot::RootRelative => String::from(r"\"),
    #[cfg(target_family = "windows")]
    InputRoot::Disk(drive) => format!("{}:\\", char::from(drive)),
    #[cfg(target_family = "windows")]
    InputRoot::DriveRelative(drive) => format!("{}:", char::from(drive)),
  };

  for (index, component) in components.iter().enumerate() {
    if index > 0 {
      expected.push_str(native_separator());
    }
    match component {
      NormalizedAtom::Name(name) => expected.push_str(name),
      NormalizedAtom::Parent => expected.push_str(".."),
    }
  }

  if components.is_empty() {
    match case.root {
      InputRoot::Relative => expected.push('.'),
      #[cfg(target_family = "windows")]
      InputRoot::DriveRelative(_) => expected.push('.'),
      #[cfg(target_family = "unix")]
      InputRoot::Unix => {}
      #[cfg(target_family = "windows")]
      InputRoot::RootRelative | InputRoot::Disk(_) => {}
    }
  }

  if has_trailing_separator(input)
    && (!components.is_empty() || !case.root.is_rooted())
    && !expected.ends_with(native_separator())
  {
    expected.push_str(native_separator());
  }
  PathBuf::from(expected)
}

fn has_trailing_separator(input: &str) -> bool {
  #[cfg(target_family = "unix")]
  {
    input.ends_with('/')
  }
  #[cfg(target_family = "windows")]
  {
    input.ends_with(['/', '\\'])
  }
}

#[test]
fn public_normalize_matches_the_bounded_independent_model() {
  let cases = generated_cases();
  let mut comparisons = 0;

  for mode in MODES {
    for case in &cases {
      let input = render_input(case, *mode);
      let expected = expected_normalized(case, &input);
      let receiver = Path::new(&input);
      let non_consuming = receiver.normalize();
      let consumed = PathBuf::from(&input).into_normalized();
      comparisons += 1;

      assert_eq!(
        non_consuming.as_os_str(),
        expected.as_os_str(),
        "non-consuming: platform={}; mode={}; case={case:?}; input={input:?}; expected={:?}; actual={:?}",
        platform_name(),
        mode.name,
        expected.as_os_str(),
        non_consuming.as_os_str(),
      );

      let input_is_expected = receiver.as_os_str() == expected.as_os_str();
      let expected_is_static_dot = expected.as_os_str() == Path::new(".").as_os_str();
      match non_consuming {
        Cow::Borrowed(actual) => {
          assert!(
            input_is_expected || expected_is_static_dot,
            "unexpected borrow: platform={}; mode={}; case={case:?}; input={input:?}; actual={:?}",
            platform_name(),
            mode.name,
            actual.as_os_str(),
          );
          if input_is_expected {
            assert!(
              std::ptr::eq(actual, receiver),
              "borrow source: platform={}; mode={}; case={case:?}; input={input:?}",
              platform_name(),
              mode.name,
            );
          }
        }
        Cow::Owned(_) => assert!(
          !input_is_expected && !expected_is_static_dot,
          "unexpected allocation: platform={}; mode={}; case={case:?}; input={input:?}; expected={:?}",
          platform_name(),
          mode.name,
          expected.as_os_str(),
        ),
      }
      assert_eq!(
        consumed.as_os_str(),
        expected.as_os_str(),
        "consuming: platform={}; mode={}; case={case:?}; input={input:?}; expected={:?}; actual={:?}",
        platform_name(),
        mode.name,
        expected.as_os_str(),
        consumed.as_os_str(),
      );
    }
  }

  assert_eq!(comparisons, EXPECTED_CASES);
}
