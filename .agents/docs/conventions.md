# Testing conventions

## Exact path spelling is observable

When a test specifies emitted spelling, compare the result through `assert_eq_str!` or compare its `OsStr` representation exactly. Standard `Path` equality can hide distinctions that this crate deliberately exposes, including a trailing separator and Windows drive-letter case. Use direct `Path` equality only when the test is about standard path identity rather than SugarPath's exact output.

Public normalization is idempotent in encoded representation, but it does not promise one spelling for every pair of values that `Path` considers equal. It preserves one trailing separator on a non-root path and preserves the input spelling of Windows drive letters. For example, `foo` and `foo/` can compare equal as `Path` values while normalizing to different exact spellings. Keep both the trailing-separator and drive-spelling cases in [`tests/path_identity.rs`](../../tests/path_identity.rs).

Resolution has a separate exact-output rule. Absolutization and relative calculation strip non-root trailing separators, and equal relative inputs return an empty path rather than the `.` emitted by normalizing an empty path. Test these operations independently instead of deriving their expected spelling by calling public `normalize`.

Invalid native encoding must be checked at the platform representation level. Unix tests compare raw bytes and Windows tests compare wide code units so a stable but lossy value cannot satisfy a test accidentally. Exercise normalization, relative fallback, strict and lossy slash conversion, and absolutization before changing encoded-byte comparison or reconstruction code.

## Allocation behavior needs explicit coverage

For `Cow`-returning APIs, output equality is necessary but insufficient. Assert the intended variant for every allocation-sensitive class. An unchanged `normalize` result normally borrows its receiver; an input that reduces to `.` can instead borrow the static current-directory path. A canonical descendant `relative` result can borrow an exact suffix of the target receiver, while upward, dirty, invalid-encoding, and different-root results are normally owned. The lifetime must never come from the base or explicit cwd.

For consuming methods, verify storage reuse as well as output. `into_normalized` and valid-Unicode owned slash conversion should retain the input allocation on their clean paths. `try_into_slash` must return the original `PathBuf`, including its native encoding, when conversion fails. Strict, fallible, and lossy methods must have the same separator semantics on valid Unicode and distinct invalid-Unicode behavior.

Explicit-cwd tests should cover both borrowed and owned inputs. A moved `PathBuf` is allowed to become the resolution buffer, while the result lifetime remains tied only to the receiver. Also cover cwd-independent operations with an invalid unused cwd to preserve lazy validation.

The checked-in allocation snapshots hard-gate successful `alloc`, `alloc_zeroed`, and `realloc` call counts for warmed named operations. Requested bytes remain target-specific evidence and may change without failing the check. The continuous matrix is deliberately small: Linux x86_64 GNU and Windows x86_64 MSVC under the Rolldown (`cached_current_dir`) configuration only. Default-feature and macOS results may still be printed or written locally for investigation, but they are not committed CI gates. When an intended optimization changes counts, regenerate the affected committed snapshot(s) and explain which execution path changed. Do not update a snapshot merely to make CI green.

Pair an allocation-sensitive implementation change with the matching timing workload. Allocation counts state how often the allocator is crossed, while benchmarks determine whether an extra classification scan or other CPU work is worthwhile for Rolldown's observed hit rate.

## Encoding policy needs explicit coverage

Slash conversion has three caller-visible policies. Strict `to_slash` and `into_slash` panic on invalid Unicode, `try_to_slash` and `try_into_slash` preserve failure without replacement, and only `to_slash_lossy` and `into_slash_lossy` insert replacement characters. Cover borrowed `Path`, known-UTF-8 `str`, and consuming `PathBuf` separately because they have different borrowing and recovery opportunities.

Slash conversion changes only the target platform's main separator. It does not normalize `.`, `..`, repeated separators, or trailing separators, and it does not parse foreign-platform syntax. Tests for a relative path followed by strict slash conversion should compare the ordinary composition rather than assume a hidden normalization step.

## Platform-specific cases stay platform-gated

Write Unix and Windows spelling expectations under compile-time `cfg` gates. `std::path` interprets a string according to the host target, so running Windows-looking inputs through Unix semantics does not validate Windows behavior. Keep shared invariant tests outside the gates, and rely on the CI operating-system matrix for target-specific execution.

Windows relative tests must distinguish ordinary drive, verbatim drive, ordinary UNC, verbatim UNC, device namespace, and generic verbatim roots. Within the same namespace, cover ASCII case differences and at least one non-ASCII case pair to prove that comparison is ASCII-insensitive rather than Unicode-folding. Drive-relative tests need ambient per-drive resolution, explicit same-drive resolution, explicit different-drive preservation, and cases where target and base can or cannot cancel unknown shared context.

Platform gating must also cover imports and helper uses. CI treats warnings as errors on every target, so an import used only by a Unix test can fail the Windows build even when the algorithm is correct; commit [`5eec2b0`](https://github.com/hyf0/sugar_path/commit/5eec2b0537833b61b5a471413ee82b9e8393c701) records one such correction.

## Local Docker validation is opt-in

The pinned Docker/Wine environment in [`benchmarks/windows-gnu.md`](../../benchmarks/windows-gnu.md) remains a reproducibility record, not a default agent action. Local Docker execution requires both a previously detected installation and an explicit developer or maintainer request for Docker in the current task; broad validation or cross-platform requests are insufficient. Without both, do not run Docker, install or start it, pull images, create volumes, or launch containers. Prefer the Windows CI matrix and state any residual local coverage gap precisely.

## Deep-path coverage must select the intended execution path

A path with more than eight components does not by itself exercise `SmallVec`: clean normalization returns early, and the common absolute relative-path path uses string slices. When changing component storage, keep a case above the relevant inline capacity that also contains `.` or `..` so `relative_str_slow` or `normalize_parts` actually runs. Keep general deep fixtures as semantic depth coverage, not as proof of a particular allocation path.

## Durable evidence

- [Shared assertion macros](../../tests/test_utils.rs)
- [Normalization output and Cow tests](../../tests/normalize.rs)
- [Exact normalization identity tests](../../tests/path_identity.rs)
- [Borrowed relative-path tests](../../tests/relative_borrowing.rs)
- [Relative lexical and drive-context tests](../../tests/relative_lexical.rs)
- [Windows roots and ASCII-only comparison tests](../../tests/relative.rs)
- [Slash encoding-policy tests](../../tests/to_slash.rs)
- [Consuming ownership tests](../../tests/owned_api.rs)
- [Invalid native encoding tests](../../tests/invalid_encoding.rs)
- [Deep-path boundary tests](../../tests/deep_paths.rs)
- [Allocation tracker and snapshot rules](../../tasks/track_allocations/README.md)
- [CI lint and test matrix](../../.github/workflows/test.yaml)
