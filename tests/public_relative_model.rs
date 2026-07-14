#![cfg(any(target_family = "unix", target_family = "windows"))]

use std::path::{Path, PathBuf};

use sugar_path::SugarPath;

const FULL_ATOMS: &[Atom] =
  &[Atom::Name("a"), Atom::Name("A"), Atom::Name("b"), Atom::Current, Atom::Parent];
const STACK_ATOMS: &[Atom] = &[Atom::Name("a"), Atom::Current, Atom::Parent];

#[cfg(target_family = "unix")]
const EXPECTED_COMPARISONS: usize = 40_368;
#[cfg(target_family = "windows")]
const EXPECTED_COMPARISONS: usize = 161_472;

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
}

#[derive(Clone, Debug)]
struct InputCase {
  root: InputRoot,
  atoms: Vec<Atom>,
}

#[derive(Clone, Copy, Debug)]
enum ResolvedRoot {
  #[cfg(target_family = "unix")]
  Unix,
  #[cfg(target_family = "windows")]
  Disk(u8),
}

#[derive(Clone, Debug)]
struct ResolvedPath {
  root: ResolvedRoot,
  components: Vec<&'static str>,
}

#[derive(Clone, Copy, Debug)]
enum Spelling {
  Clean,
  Dirty,
}

impl Spelling {
  fn separator(self) -> &'static str {
    match self {
      Self::Clean => native_separator(),
      Self::Dirty => redundant_separator(),
    }
  }

  fn has_trailing_separator(self) -> bool {
    matches!(self, Self::Dirty)
  }
}

#[derive(Clone, Copy, Debug)]
struct Mode {
  name: &'static str,
  target: Spelling,
  base: Spelling,
}

const MODES: &[Mode] = &[
  Mode { name: "clean", target: Spelling::Clean, base: Spelling::Clean },
  Mode { name: "dirty_target", target: Spelling::Dirty, base: Spelling::Clean },
  Mode { name: "dirty_base", target: Spelling::Clean, base: Spelling::Dirty },
];

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

fn roots() -> &'static [InputRoot] {
  #[cfg(target_family = "unix")]
  {
    &[InputRoot::Relative, InputRoot::Unix]
  }
  #[cfg(target_family = "windows")]
  {
    &[InputRoot::Relative, InputRoot::RootRelative, InputRoot::Disk(b'C'), InputRoot::Disk(b'D')]
  }
}

fn generated_atom_sequences() -> Vec<Vec<Atom>> {
  let mut sequences = Vec::new();
  for depth in 0..=2 {
    append_sequences(&mut sequences, FULL_ATOMS, depth);
  }
  append_sequences(&mut sequences, STACK_ATOMS, 3);
  sequences
}

fn append_sequences(sequences: &mut Vec<Vec<Atom>>, alphabet: &[Atom], depth: u32) {
  for mut ordinal in 0..alphabet.len().pow(depth) {
    let mut sequence = Vec::with_capacity(depth as usize);
    for _ in 0..depth {
      sequence.push(alphabet[ordinal % alphabet.len()]);
      ordinal /= alphabet.len();
    }
    sequences.push(sequence);
  }
}

fn generated_cases() -> Vec<InputCase> {
  generated_atom_sequences()
    .into_iter()
    .flat_map(|atoms| {
      roots().iter().map(move |root| InputCase { root: *root, atoms: atoms.clone() })
    })
    .collect()
}

fn dirty_cwd_case() -> InputCase {
  InputCase {
    #[cfg(target_family = "unix")]
    root: InputRoot::Unix,
    #[cfg(target_family = "windows")]
    root: InputRoot::Disk(b'C'),
    atoms: vec![Atom::Name("root"), Atom::Name("x"), Atom::Parent, Atom::Name("y")],
  }
}

fn render_input(case: &InputCase, spelling: Spelling) -> String {
  let mut rendered = match case.root {
    InputRoot::Relative => String::new(),
    #[cfg(target_family = "unix")]
    InputRoot::Unix => String::from("/"),
    #[cfg(target_family = "windows")]
    InputRoot::RootRelative => String::from(r"\"),
    #[cfg(target_family = "windows")]
    InputRoot::Disk(drive) => format!("{}:\\", char::from(drive)),
  };
  rendered.push_str(
    &case.atoms.iter().map(|atom| atom.spelling()).collect::<Vec<_>>().join(spelling.separator()),
  );
  if spelling.has_trailing_separator() && !case.atoms.is_empty() {
    rendered.push_str(spelling.separator());
  }
  rendered
}

fn resolve_cwd(case: &InputCase) -> ResolvedPath {
  let root = match case.root {
    InputRoot::Relative => panic!("the modeled cwd must be absolute"),
    #[cfg(target_family = "unix")]
    InputRoot::Unix => ResolvedRoot::Unix,
    #[cfg(target_family = "windows")]
    InputRoot::RootRelative => panic!("the modeled Windows cwd must include a disk"),
    #[cfg(target_family = "windows")]
    InputRoot::Disk(drive) => ResolvedRoot::Disk(drive),
  };
  let mut resolved = ResolvedPath { root, components: Vec::new() };
  apply_atoms(&mut resolved.components, &case.atoms);
  resolved
}

fn resolve_input(case: &InputCase, cwd: &ResolvedPath) -> ResolvedPath {
  let mut resolved = match case.root {
    InputRoot::Relative => cwd.clone(),
    #[cfg(target_family = "unix")]
    InputRoot::Unix => ResolvedPath { root: ResolvedRoot::Unix, components: Vec::new() },
    #[cfg(target_family = "windows")]
    InputRoot::RootRelative => ResolvedPath { root: cwd.root, components: Vec::new() },
    #[cfg(target_family = "windows")]
    InputRoot::Disk(drive) => {
      ResolvedPath { root: ResolvedRoot::Disk(drive), components: Vec::new() }
    }
  };
  apply_atoms(&mut resolved.components, &case.atoms);
  resolved
}

fn apply_atoms(components: &mut Vec<&'static str>, atoms: &[Atom]) {
  for atom in atoms {
    match *atom {
      Atom::Name(name) => components.push(name),
      Atom::Current => {}
      Atom::Parent => {
        components.pop();
      }
    }
  }
}

fn roots_equal(left: ResolvedRoot, right: ResolvedRoot) -> bool {
  #[cfg(target_family = "unix")]
  {
    matches!((left, right), (ResolvedRoot::Unix, ResolvedRoot::Unix))
  }
  #[cfg(target_family = "windows")]
  {
    match (left, right) {
      (ResolvedRoot::Disk(left), ResolvedRoot::Disk(right)) => left.eq_ignore_ascii_case(&right),
    }
  }
}

fn components_equal(left: &str, right: &str) -> bool {
  #[cfg(target_family = "unix")]
  {
    left == right
  }
  #[cfg(target_family = "windows")]
  {
    left.eq_ignore_ascii_case(right)
  }
}

fn expected_relative(target: &ResolvedPath, base: &ResolvedPath) -> PathBuf {
  if !roots_equal(target.root, base.root) {
    return render_resolved_absolute(target);
  }

  let common = target
    .components
    .iter()
    .zip(&base.components)
    .take_while(|(target, base)| components_equal(target, base))
    .count();
  let components = std::iter::repeat_n("..", base.components.len() - common)
    .chain(target.components[common..].iter().copied())
    .collect::<Vec<_>>();
  PathBuf::from(components.join(native_separator()))
}

fn render_resolved_absolute(path: &ResolvedPath) -> PathBuf {
  let suffix = path.components.join(native_separator());
  match path.root {
    #[cfg(target_family = "unix")]
    ResolvedRoot::Unix => {
      if suffix.is_empty() {
        PathBuf::from("/")
      } else {
        PathBuf::from(format!("/{suffix}"))
      }
    }
    #[cfg(target_family = "windows")]
    ResolvedRoot::Disk(drive) => PathBuf::from(format!("{}:\\{suffix}", char::from(drive))),
  }
}

#[test]
fn public_relative_with_matches_the_bounded_independent_model() {
  let cases = generated_cases();
  let cwd_case = dirty_cwd_case();
  let cwd = render_input(&cwd_case, Spelling::Clean);
  let resolved_cwd = resolve_cwd(&cwd_case);
  let resolved_cases =
    cases.iter().map(|case| resolve_input(case, &resolved_cwd)).collect::<Vec<_>>();
  let mut comparisons = 0;

  for mode in MODES {
    let rendered_targets =
      cases.iter().map(|case| render_input(case, mode.target)).collect::<Vec<_>>();
    let rendered_bases = cases.iter().map(|case| render_input(case, mode.base)).collect::<Vec<_>>();

    for (target_index, target) in rendered_targets.iter().enumerate() {
      for (base_index, base) in rendered_bases.iter().enumerate() {
        let expected =
          expected_relative(&resolved_cases[target_index], &resolved_cases[base_index]);
        let actual = Path::new(target).relative_with(Path::new(base), Path::new(&cwd));
        comparisons += 1;
        assert_eq!(
          actual.as_os_str(),
          expected.as_os_str(),
          "platform={}; mode={}; target={target:?}; base={base:?}; cwd={cwd:?}; expected={:?}; actual={:?}",
          platform_name(),
          mode.name,
          expected.as_os_str(),
          actual.as_os_str(),
        );
      }
    }
  }

  assert_eq!(comparisons, EXPECTED_COMPARISONS);
}
