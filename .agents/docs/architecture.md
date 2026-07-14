# Architecture

## Extension traits over standard types

`SugarPath` is a sealed borrowed-operation namespace implemented directly for `Path` and `str`. `PathBuf`, `String`, and other owned standard types receive the methods through normal deref lookup. The library keeps the full native `Path` domain, including non-UTF-8 paths, and does not introduce a wrapper type or a caller-selected Windows or Unix parser.

The borrowed surface is `normalize`; ambient `absolutize`, `try_absolutize`, `relative`, and `try_relative`; explicit-context `absolutize_with` and `relative_with`; strict, fallible, and lossy `to_slash`, `try_to_slash`, and `to_slash_lossy`; and `as_path`. `normalize`, absolutization, and relative calculation return `Cow<Path>`, while slash conversion returns `Cow<str>`.

`Path` owns the native path algorithms. The `str` implementation delegates path operations to `Path` but handles slash conversion directly, because the input is already known to be UTF-8 and an unchanged result can retain the input slice lifetime.

`SugarPathBuf` is a separate sealed trait implemented only for `PathBuf`. Its complete surface is `into_normalized`, strict `into_slash`, fallible `try_into_slash`, and explicitly lossy `into_slash_lossy`. It contains only operations for which consuming ownership can reuse storage; there are no mechanical consuming counterparts for relative calculation or absolutization.

Earlier `Cow`-input, separate borrowed-relative, and fused relative-to-string API experiments are superseded by this surface. Their benchmark history remains useful in [Performance strategy](./performance-strategy.md), but their method shapes are not current API.

## Public surface and implementation boundary

The root [`README`](../../README.md) is the task-oriented entry point: it helps users select a method family and states the lexical, host-native, cwd, encoding, and ownership boundaries without repeating every platform corner case. [`src/lib.rs`](../../src/lib.rs) is the docs.rs landing page and presents the same mental model through intra-doc links. [`src/sugar_path.rs`](../../src/sugar_path.rs) and [`src/sugar_path_buf.rs`](../../src/sugar_path_buf.rs) are authoritative for each method's borrowing, error, panic, and platform contract. [`src/impl_sugar_path.rs`](../../src/impl_sugar_path.rs) owns algorithms, fast paths, platform branches, and private outcome types. Private fused work is allowed when it improves the implementation, but it does not create another public method unless a consumer benchmark justifies a distinct caller-visible contract.

## Allocation is part of the contract

`normalize` and `relative` return `Cow<Path>` so caller-visible ownership follows the work performed. An already normalized path can be borrowed, and a clean descendant relative result can borrow the exact suffix of the target receiver. A clean equal absolute pair can borrow the receiver's empty suffix, while other execution paths can own the empty result. Callers that require `PathBuf` explicitly call `into_owned`. Dirty, upward, differently rooted, or otherwise rebuilt relative results are owned.

`SugarPathBuf` exposes the ownership-transfer cases directly. `into_normalized` keeps or rewrites the receiver's buffer when possible. `into_slash` and `into_slash_lossy` reuse valid-UTF-8 `PathBuf` storage, and `try_into_slash` returns the original `PathBuf` on invalid Unicode instead of destroying the native value.

`absolutize_with` and `relative_with` accept `impl AsRef<Path> + Into<PathBuf>`. A borrowed cwd remains convenient, while an owned `PathBuf` can be moved into resolution and its allocation reused. The returned `Cow` is always tied to the receiver; it never borrows the cwd argument.

String conversion makes encoding policy explicit. Strict `to_slash` and `into_slash` return the successful value directly and panic for invalid Unicode. `try_to_slash` returns `None`, and `try_into_slash` returns the original `PathBuf`, without replacement. Only the methods named `lossy` insert Unicode replacement characters.

The borrowed-versus-owned distinction and allocation counts need tests because an ownership regression can add an allocation to every Rolldown path operation even when output text is unchanged.

## Execution paths

Normalization first classifies whether rebuilding is required, then resolves `.`, `..`, and redundant native separators lexically. Public `normalize` uses trailing-separator preservation: one trailing separator survives on a non-root normalized path. Internal resolution normalization used by absolutization and relative calculation strips trailing separators, so equal relative inputs return an empty path and a descendant such as `/a/b/` relative to `/a` returns `b`.

Relative calculation first asks whether the two inputs determine the result without cwd. Absolute inputs and plain relative inputs with matching unresolved leading-parent counts can do so; Windows drive-relative inputs also have a same-drive cancellation path. Clean absolute UTF-8 descendants use direct scans and can retain a receiver suffix. Dirty spelling, upward relations, invalid native encoding, and cases that need resolution use component-based fallbacks. These fast paths preserve the public semantics rather than define a second set of semantics.

Ambient `try_absolutize` and `try_relative` read the process cwd only after cwd-independent paths fail, and their non-`try` counterparts panic only when that required lookup fails. Explicit `absolutize_with` and `relative_with` never read ambient cwd and validate that the supplied cwd is absolute only when the operation needs it.

## Platform behavior stays behind compile-time branches

SugarPath follows the compilation target's `std::path` parsing and native separator rules. Unix and Windows differences stay in target-gated helpers and tests instead of being inferred from the spelling of input at runtime.

Windows normalization emits native separators but preserves the input spelling of ordinary and verbatim drive letters. A `/` inside a verbatim path is a literal component character under `std::path`, not a separator, so normalization preserves it exactly. Removing lexical components can expose a normal value such as `C:foo` at byte zero, where reparsing would turn it into a drive prefix; normalization keeps or inserts the minimal `.\` in that case. Relative root and component comparison ignores ASCII case only; it does not perform Unicode case folding. Ordinary drives, verbatim drives, ordinary UNC, verbatim UNC, device namespaces, and generic verbatim namespaces remain distinct root kinds even when their visible names match. Within one namespace, drive, server, share, device, and path-component comparisons use the same ASCII-only rule, including invalid native encoding.

For ordinary UNC paths, SugarPath intentionally differs from Node when the server matches but the share differs. The UNC root is `server + share`, so no relative spelling can cross to the target share and SugarPath returns the normalized target. The same full-root rule applies to verbatim UNC, while ordinary and verbatim UNC remain separate namespaces.

A same-root Windows relation can also be unrepresentable as a standalone relative `Path`. A verbatim target component containing literal `/` would lose its meaning outside the verbatim prefix, and a leading normal component such as `C:foo` would be reparsed as a drive prefix. These cases return the resolution-normalized target rather than emit a different path. It is normally absolute after full resolution, but can remain root-relative or drive-relative when the inputs cancel a shared unknown context.

Windows root-relative paths share an unknown current drive, so two such inputs can relate without reading cwd. Drive-relative paths carry per-drive cwd context. Ambient absolutization delegates that resolution to `std::path::absolute` and then preserves the input drive-letter spelling. With an explicit cwd on the same drive, the cwd can resolve the input and the result still uses the input drive spelling. An explicit cwd on another drive contains no information about the input drive's remembered cwd, so the normalized drive-relative value is returned instead of fabricating a root or reading ambient state. `relative_with` can still cancel this unknown context when target and base have the same drive and the same unresolved leading-parent count; otherwise it returns the normalized target.

Native Linux x86_64 GNU, Windows x86_64 MSVC, and macOS ARM64 are the continuous integration targets. Each job verifies its Rust host before testing, macOS additionally requires the `neon` target feature, and each platform requires a registered target-specific sentinel so a `cfg` change cannot silently turn the relevant coverage into zero tests. Windows GNU remains an opt-in historical reproduction target rather than a continuous CI target. A shared-code change is incomplete if it only reasons from the developer host's interpretation of path strings.

## Current-directory access is isolated

[`src/utils.rs`](../../src/utils.rs) is the only ambient current-directory provider. Absolute and otherwise cwd-independent operations bypass it. The `cached_current_dir` feature changes the provider to process-lifetime `OnceLock` state without spreading feature checks through path algorithms. Drive-relative ambient resolution remains authoritative Windows behavior and does not substitute this single cached cwd for another drive's remembered cwd.

## Durable evidence

- [Borrowed trait contract](../../src/sugar_path.rs)
- [Consuming trait contract](../../src/sugar_path_buf.rs)
- [Implementation and private helpers](../../src/impl_sugar_path.rs)
- [Owned reuse and conversion tests](../../tests/owned_api.rs)
- [Borrowed relative-path tests](../../tests/relative_borrowing.rs)
- [Relative lexical and explicit-cwd tests](../../tests/relative_lexical.rs)
- [Ambient cwd avoidance tests](../../tests/relative_without_cwd.rs)
- [Windows root and comparison tests](../../tests/relative.rs)
- [Current-directory provider](../../src/utils.rs)
- [Cross-platform CI](../../.github/workflows/test.yaml)
