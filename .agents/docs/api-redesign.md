# Public API redesign

## Status

This record is the settled design and implementation checkpoint for the breaking SugarPath API revision. The public surface, lexical semantics, ownership behavior, native-path coverage, tests, and allocation evidence described below are implemented. Continuous allocation gates cover native Linux and Windows under the Rolldown (`cached_current_dir`) configuration; other hosts and the public default feature set remain local diagnostics. Backward compatibility and migration cost were not design constraints; unpublished branch prototypes were rejected instead of being carried into the coherent breaking surface.

## Agreed direction

### Standard Rust paths remain the domain

SugarPath must support the full `std::path::Path` and `OsStr` domain, including non-UTF-8 native paths. It follows the compilation target's host-native `std::path` parsing and separator rules rather than parsing a caller-selected Windows or POSIX syntax. UTF-8-heavy consumers such as Rolldown justify optimized paths and ergonomic strict string conversion, but they do not narrow the library's semantic domain.

`SugarPath` remains a sealed extension-method namespace implemented directly for `Path` and `str`. The `str` implementation should preserve and exploit its known-UTF-8 input instead of mechanically discarding that information through `Path` for string conversion. Owned standard types continue to reach the borrowed operations through normal deref method lookup. `SugarPathBuf` remains a separate sealed trait implemented only for `PathBuf`, and contains only operations for which consuming ownership has measured value.

### Relative paths use cwd when the inputs require it

The receiver remains the target: call `target.relative(base)`. Relative calculation is lexical and Node-style within host-native `std::path` semantics: normalize `.` and `..`, collapse redundant separators, return an empty path for equal inputs, and return the normalized absolute target when Windows roots cannot be crossed. It does not inspect the filesystem, resolve symlinks, or preserve redundant input spelling. Unlike `normalize`, Node-style `relative` does not preserve a target's trailing separator: `/a/b/` relative to `/a` is `b`, equal paths return an empty path, and upward results such as `..` have no trailing separator. A descendant fast path may still borrow the receiver slice with its final separator excluded.

`relative` may read the process current directory when relative inputs do not determine an answer by themselves. It should retain cwd-independent fast paths for absolute inputs and relative shapes whose common cwd cancels. A companion operation accepts an explicit external cwd, uses it to resolve both target and base, and never reads the process cwd. Given the same complete cwd context, the ambient and explicit variants return the same path.

`relative` returns `Cow<Path>` directly. A clean descendant may borrow the exact normalized suffix from the receiver after a classification scan proves that no transformation is required; upward, dirty, differently rooted, or otherwise rebuilt results are owned. An unpublished `relative_cow` prototype established this ownership model, but the final surface exposes it through `relative` itself; callers requiring a `PathBuf` call `Cow::into_owned`.

Node is the behavioral reference for host-native normalization, resolution, and relative calculation where its output represents a valid native path relation. SugarPath deliberately differs for ordinary Windows UNC paths whose server is equal but share is different: Node emits a relative spelling that resolves inside the original share rather than reaching the target share. SugarPath treats `server + share` as the UNC root and returns the normalized absolute target when shares differ, matching Windows and Rust path structure.

Following Node also changes two current normalization contracts: preserve one trailing separator on a non-root normalized path, and preserve the input spelling of a Windows drive letter instead of uppercasing it. The trailing rule includes collapsing `foo//` to `foo/` and reducing `foo/../` to `./`, while roots remain roots. Drive spelling is preserved for both ordinary disk and verbatim-disk prefixes, but drive/root comparison remains case-insensitive, so `C:\a` and `c:\a\b` share a root and relate as `b`. Rust-only native encodings and prefix forms without a Node string equivalent continue to follow `std::path`.

Windows drive-relative input is valid native path state, not an error. Ambient operations resolve `C:foo` through Windows' remembered cwd for drive C, matching Node and `std::path::absolute`. An explicit cwd on drive C resolves `C:foo` against that cwd. An explicit cwd on another drive does not contain C's remembered cwd, so `absolutize_with` returns the normalized drive-relative input, such as `C:foo`, without consulting ambient state or fabricating `C:\foo`. `relative_with` must calculate a relative result when the unknown shared context provably cancels: normalized drive-relative target and base have the same drive and the same unresolved leading-`..` count. For example, target `C:foo`, base `C:bar`, and explicit cwd `D:\cwd` produce `..\foo`. Otherwise, when the operation cannot relate the resolved inputs, it returns the normalized target, which may itself remain drive-relative. This is the same information-preserving fallback principle used for different roots, without claiming that every `absolutize_with` result satisfies `Path::is_absolute`.

### Default methods stay ergonomic and `try_*` exposes failure

Ordinary methods do not make the common call site handle `Result`. `absolutize` and `relative` may panic if the environment cannot provide required cwd state, with the failure documented under `# Panics`; `try_absolutize` and `try_relative` expose the underlying `io::Error`. Only ambient-cwd operations receive these `try_*` variants. The explicit-cwd variants perform no environment lookup and need no fallible counterpart. Their cwd argument represents an absolute current directory; when an operation actually needs that argument, a non-absolute cwd violates the contract and panics with a documented message. An absolute receiver, or a relative calculation whose inputs determine the answer without cwd, does not inspect or reject an unused cwd. A different-drive `C:foo` input is not a contract violation and follows the information-preserving fallback above.

This policy applies to string conversion as well. The common strict slash conversion should return its value directly and require valid UTF-8. A `try_*` form preserves non-UTF-8 input without replacement, while the explicitly named `lossy` form remains available only for callers that choose replacement characters. Rolldown's known-UTF-8 paths should not use a lossy-named operation.

The common strict method is named `to_slash`, not `expect_to_slash`; it is semantically the successful value of `try_to_slash` and panics for invalid Unicode with that contract documented. `try_to_slash` returns `None` without replacement, and `to_slash_lossy` remains the explicit replacement operation. `into_slash` likewise returns the successful value of `try_into_slash` and panics on invalid Unicode, while the failed consuming form returns the original `PathBuf`. The explicit-context operations remain `absolutize_with` and `relative_with`; their cwd parameter and panic documentation make a longer `_with_cwd` suffix unnecessary.

### Explicit cwd keeps transferable ownership without an explicit `Cow`

`absolutize_with` remains part of the API because callers can construct a temporary cwd-derived `PathBuf` and transfer that allocation into the result. Rolldown does this in both module finalization and chunk generation with `cwd.join(out_dir)`. The final implementation moves that owned cwd into the result path; the clean allocation rows record no fresh allocation and only the buffer growth required to append the receiver.

The caller should not need to spell `Cow::Owned` or `Cow::Borrowed`. The explicit cwd parameter should accept a value satisfying both `AsRef<Path>` and `Into<PathBuf>`: a borrowed path is inspected and copied only when the operation needs an owned base, while an owned `PathBuf` is moved and reused. `relative_with` should use the same cwd input contract. The returned `Cow` remains tied to the receiver and never borrows the cwd.

### Cached cwd remains available

The `cached_current_dir` feature remains part of the design. It is an explicit opt-in for processes such as Rolldown that treat cwd as stable, while `absolutize_with` and `relative_with` remain the way to supply a changing or externally managed cwd. Ambient methods must still avoid initializing or reading the cache when their inputs determine the answer without cwd. A single cached cwd cannot invent another drive's remembered cwd: ambient resolution of an otherwise unresolved `C:foo` must use the authoritative Windows behavior rather than substituting a rooted path or reusing a cwd from another drive.

### Performance requirements are recorded independently from fused method names

Rolldown's package-sideEffects path needs a temporary relative view and overwhelmingly receives clean descendants, so the public `relative -> Cow<Path>` contract must permit a zero-result-allocation hit. Stable IDs and sourcemap sources end as owned UTF-8 slash strings and should not require an avoidable intermediate allocation or a second result buffer. An unpublished `relative_to_slash_lossy` prototype tested direct output, but the final surface uses the ordinary composition `relative(base).into_owned().into_slash()`, where an owned relative result moves through and a borrowed result allocates the buffer required by the final `String`. Keep direct-output logic private and reconsider a new fused public operation only if a real final-container benchmark later proves that this composition cannot meet the allocation target.

For canonical native-spelling descendant inputs, the target allocation counts are zero for `relative -> Cow<Path>` on both Unix and Windows, and exactly the final output allocation for `Cow::into_owned` or the ordinary final-`String` composition. The Windows implementation must not allocate normalized copies of target and base merely to compare separators. Clean `PathBuf::into_normalized` and valid-Unicode owned slash conversion retain their existing zero-allocation targets, and an owned temporary cwd passed to `absolutize_with` must be reusable without the current post-normalization clone. Dirty, invalid-encoding, different-root, and upward cases keep separate measurements rather than being forced into the clean-path count.

## Target public surface

The method roles, names, and error policy below are the implementation target.

```rust
fn normalize(&self) -> Cow<'_, Path>;

fn absolutize(&self) -> Cow<'_, Path>;
fn try_absolutize(&self) -> io::Result<Cow<'_, Path>>;
fn absolutize_with(
  &self,
  cwd: impl AsRef<Path> + Into<PathBuf>,
) -> Cow<'_, Path>;

fn relative(&self, base: impl AsRef<Path>) -> Cow<'_, Path>;
fn try_relative(&self, base: impl AsRef<Path>) -> io::Result<Cow<'_, Path>>;
fn relative_with(
  &self,
  base: impl AsRef<Path>,
  cwd: impl AsRef<Path> + Into<PathBuf>,
) -> Cow<'_, Path>;

fn to_slash(&self) -> Cow<'_, str>;
fn try_to_slash(&self) -> Option<Cow<'_, str>>;
fn to_slash_lossy(&self) -> Cow<'_, str>;
fn as_path(&self) -> &Path;
```

The consuming `PathBuf` surface is limited to operations where ownership has measured value:

```rust
fn into_normalized(self) -> PathBuf;
fn into_slash(self) -> String;
fn try_into_slash(self) -> Result<String, PathBuf>;
fn into_slash_lossy(self) -> String;
```

`try_into_slash` returns the original `PathBuf` on invalid UTF-8 so a failed consuming conversion does not destroy the native path. Do not add `into_relative`, `into_absolutize`, or a mechanical consuming counterpart for every borrowed operation.

## Final contract notes

- Only ambient `absolutize` and `relative` receive `try_*` variants; explicit-cwd methods never read ambient state and validate their cwd only when the result actually depends on it.
- `as_path` remains the allocation-free standard-library view used by string receivers; it does not create another path representation.
- A valid Windows drive-relative input on a different drive is preserved in normalized drive-relative form rather than treated as an error.
- Windows normalization keeps or inserts the minimal `.\` when its absence would reinterpret the first normal component as a drive prefix.
- A Windows target whose remaining components cannot be represented as a standalone native relative `Path` returns the resolution-normalized target; this includes verbatim components containing literal `/` and a leading normal component that would be reparsed as a prefix. The fallback is normally absolute but can remain root-relative or drive-relative when the inputs cancel a shared unknown context.
- `normalize` preserves one trailing separator, while `relative` removes target trailing separators as part of Node-style resolution.
- Public method names describe caller-visible semantics. Private outcome types and direct-output implementations may remain fused internally, but allocation mechanisms do not create additional public methods.

## Implementation status

- [x] Finalize the method matrix, lifetimes, panic sections, explicit-cwd fallback, and `try_*` error policy as one API review before changing implementation.
- [x] Add contract tests for ambient and explicit cwd equivalence, lazy cwd access, cwd lookup failure, explicit non-absolute cwd, equal relative output, normalization-versus-relative trailing separators, Windows different roots and UNC shares, and Windows drive-relative inputs with same-drive, different-drive, and cancellable unknown contexts.
- [x] Change `relative` to return `Cow<Path>`, add the ambient `try_*` and explicit-cwd behavior selected by the review, and leave the unpublished `relative_cow` prototype out of the final surface.
- [x] Align `absolutize`, its `try_*` form, and `absolutize_with` with the same context and error policy; replace the explicit `Cow<Path>` parameter while preserving owned temporary-cwd reuse.
- [x] Implement `into_normalized` plus strict, fallible strict, and explicitly lossy slash conversion for borrowed and consuming inputs while preserving arbitrary native encoding outside string conversion.
- [x] Benchmark package-sideEffects, stable ID, sourcemap, owned `PathBuf`, and native-separator paths against final containers using the ordinary consuming composition, then leave the unpublished fused prototype out of the final surface.
- [x] Update README examples, rustdoc, changelog, native allocation snapshots, and PCR records that described earlier branch prototypes.
- [x] Replace the historical multi-host dual-config allocation matrix with final-API continuous gates: Linux x86_64 GNU and Windows x86_64 MSVC under the Rolldown configuration, produced and checked on native GitHub Actions.

Delivery order is part of the evidence: baseline PR #41 landed on main first as `9e6b627`, with the same tree as reviewed baseline tip `e483f8f`, and PR #40 is rebuilt on top of it. Benchmark rows keep receiver/input-shape and final-output identifiers stable across that boundary; historical `9712b6e` and Windows-GNU measurements retain their original labels rather than being presented as #41 results.

Rolldown source migration is intentionally outside this repository change. After the breaking SugarPath release, migrate known-UTF-8 package-sideEffects, stable ID, and sourcemap call sites while preserving each caller's handling of an empty relative result and rechecking the package-sideEffects pre-scan tradeoff with consumer-weighted evidence.

## Evidence to preserve

- [Current public traits](../../src/sugar_path.rs)
- [Current consuming trait](../../src/sugar_path_buf.rs)
- [Relative cwd behavior](../../tests/relative_without_cwd.rs)
- [Relative lexical matrix](../../tests/relative_lexical.rs)
- [Invalid native encoding](../../tests/invalid_encoding.rs)
- [Rolldown workload and allocation evidence](./performance-strategy.md)
