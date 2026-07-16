# Semantic test strategy

## Goal

SugarPath treats complete semantic coverage as a finite set of behavior partitions, not as literal enumeration of the unbounded set of possible path strings. Every public method must be represented across the partitions that can change its output, error policy, ownership result, native encoding, or execution branch, and bounded deterministic generation must cover combinations inside the high-risk parsing and relative-path partitions.

## Required behavior partitions

| Dimension | Required partitions |
| --- | --- |
| Public surface | Borrowed `Path`, known-UTF-8 `str` and deref receivers, and consuming `PathBuf` methods |
| Platform | Linux and macOS Unix semantics, macOS ARM64 NEON dispatch, Windows disk, root-relative, drive-relative, UNC, verbatim, device, and generic namespace forms, plus browser WebAssembly compilation and executable WASIp1 semantics |
| Spelling | Empty, current directory, leading and interior parents, clean and dirty separators, trailing separators, roots, deep paths, and multibyte components |
| Context | Cwd-independent, ambient-cwd-dependent, explicit borrowed cwd, explicit owned cwd, unused invalid cwd, unavailable ambient cwd, and cached cwd |
| Encoding | Valid UTF-8 and native-invalid Unix bytes or Windows wide units, compared in their native representation |
| Result contract | Exact spelling, strict/fallible/lossy policy, `Cow` borrowed or owned variant and borrow source, owned-buffer reuse where promised, round trip where representable, and panic or error behavior |

The matrix is interaction-aware. A platform root kind must be crossed with encoding when root comparison consumes native encoding, and a fast path must be crossed with clean and dirty spelling when its classifier selects between borrowing and rebuilding. Unrelated dimensions do not require a blind Cartesian product when their independence is already enforced by a narrower test.

## Enforcement rules

- Literal expected-output tables pin public behavior. One public method may be compared with another to test parity, but it is not an independent oracle; the covered partition must also have a literal expectation or a separate test oracle.
- Generated tests must call the production dispatch as well as any private helper under review. A helper-only exhaustive test cannot prove that the public path still selects that helper.
- Bounded generators must assert their expected corpus or comparison count. A reduced generation depth that leaves all remaining comparisons green is still a coverage failure.
- Platform-gated coverage must not silently disappear or stop executing. CI verifies the exact Rust host for Linux, Windows, and macOS ARM64, requires `neon` on macOS, checks an explicit list of critical target-specific tests in both the default and `cached_current_dir` configurations, and rejects every ignored unit, integration, or documentation test registered by those commands.
- WebAssembly support has two separate gates: `wasm32-unknown-unknown` compiles the production library with and without cwd caching, while `wasm32-wasip1` executes selected public contracts, native byte encoding, both cwd policies, and rustdoc examples under Wasmtime. Every WASIp1-specific integration target must be named `wasm_wasi_*.rs`; CI selects that Cargo target prefix in every Clippy, test, listing, and ignored-test command, so a new matching target is included automatically. Fixed test-name assertions and ignored-test scans continue to verify the critical contracts that must register. These are correctness gates; WebAssembly is not a performance target.
- Default and `cached_current_dir` production coverage must remain distinct. CI declares the expected feature state for each test command, and an executable sentinel rejects a missing expectation or workspace feature unification.
- Exact native spelling and encoding are asserted directly. `Path` equality and lossy conversion are insufficient when trailing separators, drive spelling, or invalid encoding are observable.
- `Cow` and consuming APIs assert ownership separately from value equality. A test that checks only output text does not protect the allocation-facing contract.

## Normalization coverage checkpoint

- An independent public normalization model resolves its own root and component stack without calling SugarPath or `std::path::components`, then checks exact non-consuming and consuming results, their exact idempotence, and the non-consuming `Cow` contract for 6,560 Unix and 14,040 Windows inputs. The corpus crosses relative and rooted forms, drive-relative Windows paths, clean, redundant, forward-slash Windows and trailing separators, parents, current-directory components, dot-prefixed ordinary names, case-sensitive spelling, multibyte names, and Unix backslashes as ordinary bytes. Direct tests prove that standard `Path` equality treats `.` and the native dot-plus-separator spelling as equal while SugarPath preserves their distinct exact outputs, Windows prefix-like ordinary components remain exactly idempotent across both normalization APIs, and WASIp1 checks the same fixed-point property through raw bytes.

## Relative-path coverage checkpoint

- `relative_with` has fixed Unix and Windows expected-output matrices for relative, absolute, mixed-context, dirty, equal, root-clamped, trailing-separator, root-relative, drive-relative, and different-drive results, using both borrowed and owned cwd arguments.
- Cwd-independent rows pin the same literal result through `relative`, `try_relative`, and both explicit-cwd argument forms while proving that an unused non-absolute cwd is not validated.
- `try_relative` and `relative_with` assert borrowed descendant and equal suffixes, owned upward and dirty results, owned cwd-resolved results, and receiver-only borrowing when base and cwd are owned temporaries.
- An independent public `relative_with` model resolves its own root and component structures without calling SugarPath or `std::path::components`, then checks 40,368 Unix and 161,472 Windows combinations across clean and dirty native spellings. Its dirty absolute cwd contains `..`; Unix comparison is exact, while Windows roots and components compare with ASCII case ignored.
- On macOS and Linux, all 224,676 pairs from the bounded short absolute spelling set plus the multibyte set are checked through the production `relative_str` dispatch and the suffix-validation helper against the slow component oracle.
- Absolute paths with native-invalid normal components prove that equal raw encoding cancels and distinct encoding with the same lossy rendering does not, through `relative`, `try_relative`, and `relative_with` on Unix and Windows. Unix explicit-cwd rows also prove that relative resolution preserves native-invalid cwd components for borrowed and owned context arguments.
- Windows drive-relative rows apply the same exact comparison before shared-context cancellation and preserve invalid-wide components while resolving against explicit borrowed and owned cwd arguments.
- Unavailable Unix cwd coverage pins exact successful output and `Cow` variants for cwd-independent fallible calls, preserves the ambient error and panic checks for dependent `Path`, `str`, and `String` calls, and proves that a valid explicit cwd still succeeds.
- Known-UTF-8 `str` and `String` receivers prove exact clean-trailing and dirty normalization contracts, receiver-only borrowing, explicit and ambient relative context forwarding, and cached-cwd behavior on Unix and Windows. Unix deleted-cwd rows also prove cwd-independent success and strict cwd-dependent panics for string receivers.
- A Linux child process starts in a real cwd containing an invalid native byte and requires `absolutize`, `try_absolutize`, `relative`, and `try_relative` to preserve that byte exactly under both cwd policies. Linux supplies this executable filesystem case because macOS rejects the invalid-byte directory name; explicit-cwd tests continue to cover native-invalid Unix composition independently of the host filesystem.

## Follow-up coverage status

The audit partitions were implemented in independent test-only PRs so each contract could be reviewed and merged separately:

- Slash receiver and policy coverage, including native-invalid recovery without making its storage identity a semantic requirement: [merged PR #53](https://github.com/hyfdev/sugar_path/pull/53).
- Non-ASCII and invalid-wide Windows root identifiers across UNC, device, and generic verbatim prefixes: [merged PR #54](https://github.com/hyfdev/sugar_path/pull/54).
- Systematic `try_absolutize` parity, exact Windows root-relative results, and native-invalid ambient absolutization: [merged PR #55](https://github.com/hyfdev/sugar_path/pull/55).
- Cached-cwd relative behavior and failed-initialization retry: [merged PR #56](https://github.com/hyfdev/sugar_path/pull/56).
- Independent fixed oracles for consuming normalization across Unix, every Windows prefix kind, trailing separators, and native-invalid encoding: [merged PR #57](https://github.com/hyfdev/sugar_path/pull/57).
- Independent bounded public relative semantics across clean and dirty path shapes: [merged PR #58](https://github.com/hyfdev/sugar_path/pull/58).

## Change rule

Any change to a public contract, platform branch, native-encoding comparison, classifier, ownership path, or cwd access path must identify its affected matrix rows, add or update the closest executable evidence, and keep this record current. A new optimized branch is incomplete until a test proves that production dispatch enters it on the intended native CI target and a semantic oracle or fixed matrix covers its fallback boundary.

## Durable evidence

- [Native CI target and sentinel checks](../../.github/workflows/test.yaml)
- [WASIp1 public API contracts](../../tests/wasm_wasi_contracts.rs)
- [WASIp1 cached cwd contract](../../tests/wasm_wasi_cached_current_dir.rs)
- [Default/cached-current-directory configuration sentinel](../../tests/feature_configuration.rs)
- [Fixed explicit and fallible relative matrices](../../tests/relative_lexical.rs)
- [Independent bounded public relative model](../../tests/public_relative_model.rs)
- [Independent bounded public normalization model](../../tests/public_normalize_model.rs)
- [Relative ownership and lifetime contracts](../../tests/relative_borrowing.rs)
- [Unavailable-cwd relative behavior](../../tests/relative_without_cwd.rs)
- [Known-UTF-8 string receiver ownership](../../tests/string_receiver_contracts.rs)
- [Native-invalid ambient cwd preservation](../../tests/non_utf8_ambient_cwd.rs)
- [Native-invalid exact comparison](../../tests/invalid_encoding.rs)
- [Production dispatch and short-path oracle](../../src/impl_sugar_path.rs)
