# Gotchas

## The receiver is the target in `relative`

Call `target.relative(base)`. The method returns the lexical path that reaches the receiver from its argument, so `Path::new("/base/lib").relative("/base")` is `lib`. Equal paths return an empty path. The public parameter and documentation use `base` to keep that direction explicit; use the executable cases in [`tests/relative.rs`](../../tests/relative.rs) as the contract.

## Normalization is lexical

`normalize` resolves component spelling only. It does not access the filesystem, resolve symlinks, or preserve a distinction that would emerge only after following a symlink. Parent components above an absolute root are discarded, while excess parents on a relative path remain as `..`. Use filesystem canonicalization separately when physical identity is required.

On Windows, removing `.` or earlier components can expose a normal component such as `C:foo` at the start of the result, where `std::path` would parse it as a drive prefix. `normalize` keeps or inserts `.\` for that case so the output still represents the same components. This is an information-preserving exception to ordinary dot removal.

Public normalization preserves one trailing separator on a non-root result, matching the selected Node-style behavior. Resolution normalization is deliberately different: `absolutize`, `relative`, and their variants strip non-root trailing separators. Consequently `"a/".normalize()` ends in a separator, `target.relative(base)` does not preserve a non-root target's trailing separator, and equal relative inputs produce an empty path rather than `.`. Do not call public `normalize` internally where resolution spelling is required.

## `relative` borrows only from its receiver

The main `relative` API returns `Cow<Path>`. A clean descendant can borrow an exact suffix of the target receiver after the fast path proves that no rewriting is needed. Dirty separators or components, upward results, unresolved fallbacks, invalid encoding, and different roots normally produce owned values. Call `into_owned` when a caller requires `PathBuf`; there is no separate borrowed-relative public method.

The return lifetime is tied only to the receiver, never to `base`, so a temporary base is safe. `SugarPath` is implemented directly for `str`, allowing a helper with an `&'a str` target to return `Cow<'a, Path>`. A temporary target cannot yield a borrowed value that outlives that target, as usual for `Cow` tied to a receiver.

## Explicit cwd never lends its lifetime to the output

`absolutize_with` and `relative_with` accept `impl AsRef<Path> + Into<PathBuf>`. This lets callers pass a borrowed cwd for convenience or move an owned `PathBuf` so resolution can reuse its allocation. The returned `Cow` is still tied only to the target receiver and never borrows from the cwd, even when the final value is textually equal to it.

An explicit cwd must be absolute only when the operation needs it. An absolute receiver, or a relative pair whose common context cancels lexically, does not inspect or reject an unused cwd. When resolution does depend on the supplied cwd, a non-absolute value is a contract violation and the explicit method panics; these methods do not read ambient state and therefore have no `try_*` I/O variant.

## Ambient errors are optional at the call site

`absolutize` and `relative` return `Cow<Path>` directly for the common case. If an operation needs the process cwd and it cannot be read, these methods panic. `try_absolutize` and `try_relative` expose the same calculation as `io::Result` for callers that need to handle that environmental failure. Cwd-independent inputs succeed without reading cwd in both forms.

## The cached current directory is process-lifetime state

With `cached_current_dir`, the first operation that actually needs the ordinary process cwd initializes a `OnceLock`, and later operations reuse that path. Absolute inputs and cwd-independent relative pairs do not initialize the cache. A later `std::env::set_current_dir`, directory removal, or permission change after initialization is invisible to this provider; it continues lexical resolution from the successful snapshot without revalidating that path. Enable the feature only when the process treats cwd as stable, or use `absolutize_with` and `relative_with` for changing or externally managed cwd state.

Windows drive-relative ambient resolution is a separate case: `C:foo` needs drive C's remembered cwd, not merely the process's single cached cwd. It goes through `std::path::absolute`, so the `cached_current_dir` feature must not substitute an unrelated drive context.

## Windows roots, drives, and namespaces carry different context

- `C:\foo` is absolute, `\foo` is root-relative without a drive, and `C:foo` is drive-relative. Root-relative input can use the drive from ambient or explicit cwd; drive-relative input needs the remembered cwd for its named drive.
- `normalize` preserves the input spelling of ordinary and verbatim drive letters. Ambient and explicit same-drive resolution also preserve the drive-relative input's spelling, even when resolution must rebuild the path.
- Ambient `C:foo` resolution uses `std::path::absolute` and therefore the operating system's per-drive cwd. Explicit resolution against a cwd on drive C uses that cwd. An explicit cwd on another drive does not reveal drive C's remembered cwd, so `absolutize_with` returns normalized `C:foo` rather than reading ambient state or inventing `C:\foo`.
- `relative_with` can cancel unknown drive context when target and base are drive-relative on the same ASCII-insensitive drive and have the same count of unresolved leading `..` components. If that context cannot be cancelled, the method returns the normalized target, which may remain drive-relative.
- Different ordinary drives, UNC shares, or namespace roots cannot be crossed by a native relative path, so `relative` returns the normalized target with resolution-style trailing-separator stripping.
- Ordinary drives, verbatim drives, ordinary UNC, verbatim UNC, device namespaces, and generic verbatim namespaces are distinct. Matching visible drive, server, share, or device text does not make values from different namespace kinds relatable.
- Inside a verbatim path, `/` is a literal component character rather than a separator. If removing the verbatim prefix would reinterpret that character, or if a leading normal component would be reparsed as a Windows prefix, `relative` returns the normalized target instead of emitting a misleading relative path. The fallback can remain root-relative or drive-relative when the shared unknown context cancels.
- Root and component comparison ignores ASCII case only. It does not perform Unicode case folding, and the same encoded-byte rule is used for invalid native encoding.
- For ordinary UNC paths, SugarPath intentionally differs from Node when the server matches but the share differs. SugarPath treats `server + share` as the root and returns the normalized target because a relative path within the original share cannot reach another share. Verbatim UNC uses the same full-root rule and remains distinct from ordinary UNC.

## Slash conversion policy is explicit and does not normalize

`to_slash` and `into_slash` are strict: they return the converted value directly and panic when a native path is not valid Unicode. `try_to_slash` returns `None`, while `try_into_slash` returns the original `PathBuf`, so callers can preserve failure without replacement. Only `to_slash_lossy` and `into_slash_lossy` insert replacement characters.

All slash methods convert only the target platform's main separator. They do not resolve `.`, `..`, or repeated separators, remove trailing separators, or reinterpret foreign-platform paths. On Unix, a backslash remains an ordinary character. The `str` implementation already knows its input is UTF-8 and can borrow whenever separator replacement is unnecessary.

There is no public fused relative-to-string method. The strict owned-string pipeline is `target.relative(base).into_owned().into_slash()`, and it has the same relative semantics as the intermediate `Cow<Path>`, including an empty string for equal inputs. Private direct-output work may optimize that composition, but it must not insert public `normalize` or change root decisions.

## Durable evidence

- [Relative-path semantics and Windows root tests](../../tests/relative.rs)
- [Borrowed relative-path tests](../../tests/relative_borrowing.rs)
- [Relative lexical and drive-context tests](../../tests/relative_lexical.rs)
- [Cwd-independent relative tests](../../tests/relative_without_cwd.rs)
- [Normalization and trailing-separator tests](../../tests/normalize.rs)
- [Exact normalization identity tests](../../tests/path_identity.rs)
- [Explicit-cwd absolutization tests](../../tests/absolutize_with.rs)
- [Ambient cwd avoidance tests](../../tests/absolutize_without_cwd.rs)
- [Unavailable cwd after a successful lookup](../../tests/cwd_unavailable_after_cache.rs)
- [Slash-conversion tests](../../tests/to_slash.rs)
- [Consuming ownership tests](../../tests/owned_api.rs)
- [Relative-to-slash composition tests](../../tests/relative_to_slash.rs)
- [Current-directory provider](../../src/utils.rs)
- [Windows-GNU execution and UNC regression coverage](../../benchmarks/windows-gnu.md)
