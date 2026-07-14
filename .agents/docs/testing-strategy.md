# Semantic test strategy

## Goal

SugarPath treats complete semantic coverage as a finite set of behavior partitions, not as literal enumeration of the unbounded set of possible path strings. Every public method must be represented across the partitions that can change its output, error policy, ownership result, native encoding, or execution branch, and bounded deterministic generation must cover combinations inside the high-risk parsing and relative-path partitions.

## Required behavior partitions

| Dimension | Required partitions |
| --- | --- |
| Public surface | Borrowed `Path`, known-UTF-8 `str` and deref receivers, and consuming `PathBuf` methods |
| Platform | Linux and macOS Unix semantics, macOS ARM64 NEON dispatch, and Windows disk, root-relative, drive-relative, UNC, verbatim, device, and generic namespace forms |
| Spelling | Empty, current directory, leading and interior parents, clean and dirty separators, trailing separators, roots, deep paths, and multibyte components |
| Context | Cwd-independent, ambient-cwd-dependent, explicit borrowed cwd, explicit owned cwd, unused invalid cwd, unavailable ambient cwd, and cached cwd |
| Encoding | Valid UTF-8 and native-invalid Unix bytes or Windows wide units, compared in their native representation |
| Result contract | Exact spelling, strict/fallible/lossy policy, `Cow` borrowed or owned variant and borrow source, owned-buffer reuse where promised, round trip where representable, and panic or error behavior |

The matrix is interaction-aware. A platform root kind must be crossed with encoding when root comparison consumes native encoding, and a fast path must be crossed with clean and dirty spelling when its classifier selects between borrowing and rebuilding. Unrelated dimensions do not require a blind Cartesian product when their independence is already enforced by a narrower test.

## Enforcement rules

- Literal expected-output tables pin public behavior. One public method may be compared with another to test parity, but it is not an independent oracle; the covered partition must also have a literal expectation or a separate test oracle.
- Generated tests must call the production dispatch as well as any private helper under review. A helper-only exhaustive test cannot prove that the public path still selects that helper.
- Platform-gated coverage must not silently disappear. CI verifies the exact Rust host for Linux, Windows, and macOS ARM64, requires `neon` on macOS, and checks that a target-specific sentinel test is registered after the all-feature suite.
- Default and all-feature coverage must remain distinct. CI declares the expected `cached_current_dir` state for each test command, and an executable sentinel rejects workspace feature unification in the default run.
- Exact native spelling and encoding are asserted directly. `Path` equality and lossy conversion are insufficient when trailing separators, drive spelling, or invalid encoding are observable.
- `Cow` and consuming APIs assert ownership separately from value equality. A test that checks only output text does not protect the allocation-facing contract.

## Relative-path coverage checkpoint

- `relative_with` has fixed Unix and Windows expected-output matrices for relative, absolute, mixed-context, dirty, equal, root-clamped, trailing-separator, root-relative, drive-relative, and different-drive results, using both borrowed and owned cwd arguments.
- Cwd-independent rows pin the same literal result through `relative`, `try_relative`, and both explicit-cwd argument forms while proving that an unused non-absolute cwd is not validated.
- `try_relative` and `relative_with` assert borrowed descendant and equal suffixes, owned upward and dirty results, owned cwd-resolved results, and receiver-only borrowing when base and cwd are owned temporaries.
- On macOS and Linux, all 224,676 pairs from the bounded short absolute spelling set plus the multibyte set are checked through the production `relative_str` dispatch and the suffix-validation helper against the slow component oracle.
- Unavailable Unix cwd coverage pins exact successful output and `Cow` variants for cwd-independent fallible calls, preserves the ambient error and panic checks for dependent calls, and proves that a valid explicit cwd still succeeds.

## Follow-up coverage status

The remaining audit partitions are implemented in independent test-only PRs so each contract can be reviewed and merged separately:

- Slash receiver and policy coverage, including native-invalid recovery without making its storage identity a semantic requirement: [Draft PR #53](https://github.com/hyfdev/sugar_path/pull/53).
- Non-ASCII and invalid-wide Windows root identifiers across UNC, device, and generic verbatim prefixes: [Draft PR #54](https://github.com/hyfdev/sugar_path/pull/54).
- Systematic `try_absolutize` parity, exact Windows root-relative results, and native-invalid ambient absolutization: [Draft PR #55](https://github.com/hyfdev/sugar_path/pull/55).
- Cached-cwd relative behavior and failed-initialization retry: [merged PR #56](https://github.com/hyfdev/sugar_path/pull/56).
- Independent fixed oracles for consuming normalization across Unix, every Windows prefix kind, trailing separators, and native-invalid encoding: [merged PR #57](https://github.com/hyfdev/sugar_path/pull/57).

## Change rule

Any change to a public contract, platform branch, native-encoding comparison, classifier, ownership path, or cwd access path must identify its affected matrix rows, add or update the closest executable evidence, and keep this record current. A new optimized branch is incomplete until a test proves that production dispatch enters it on the intended native CI target and a semantic oracle or fixed matrix covers its fallback boundary.

## Durable evidence

- [Native CI target and sentinel checks](../../.github/workflows/test.yaml)
- [Default/all-feature configuration sentinel](../../tests/feature_configuration.rs)
- [Fixed explicit and fallible relative matrices](../../tests/relative_lexical.rs)
- [Relative ownership and lifetime contracts](../../tests/relative_borrowing.rs)
- [Unavailable-cwd relative behavior](../../tests/relative_without_cwd.rs)
- [Production dispatch and short-path oracle](../../src/impl_sugar_path.rs)
