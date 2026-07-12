# Windows GNU execution

This document preserves the pinned Docker/Wine environment that produced earlier Windows-GNU correctness and allocation evidence from a non-Windows host. It remains useful for reproducing those recorded checkpoints, but Wine under amd64 emulation is not a wall-time performance baseline and the old snapshots are not evidence that the final API was rerun. The accepted main baseline landed first as [`9e6b627`](https://github.com/hyf0/sugar_path/commit/9e6b6277726f25b8c2edea3efa7ad097e8606fdb) in [PR #41](https://github.com/hyf0/sugar_path/pull/41), with the same tree as reviewed branch tip [`e483f8f`](https://github.com/hyf0/sugar_path/commit/e483f8f166378d9b8cc59369b188b05db00c593a); it uses native Windows-MSVC snapshots, and PR #40 follows it. References below to `9712b6e`, GNU allocation counts, or Wine execution describe the older development checkpoint only and are not results for the accepted #41 baseline.

## Final-API evidence status

The final implementation makes the main `relative` method return `Cow<Path>` and scans clean native Windows separators, roots, and common prefixes without first allocating forward-slash copies of target and base. The native descendant target is zero allocations for `relative -> Cow<Path>` and exactly one final output allocation for `Cow::into_owned` or `relative(base).into_owned().into_slash()`. Upward, dirty, mixed-separator, different-root, UNC, verbatim, device, and invalid-wide cases remain separate correctness and allocation rows.

This refresh did not include permission to run Docker, Wine, or containers, and none of those commands was run. Non-container GNU and MSVC cross-compilation checks provide build evidence but do not execute Windows path semantics. Green GitHub Actions run [`29167568157`](https://github.com/hyf0/sugar_path/actions/runs/29167568157) executes the allocation runner natively on Windows and verifies the final x86_64-MSVC default and Rolldown snapshots. The older Windows-GNU snapshots were removed instead of being relabeled; the commands below can reproduce new GNU evidence only when the local execution gate is explicitly satisfied. Native timing remains a separate requirement for any Windows speed claim.

## Local execution gate

The commands below are an opt-in reproduction reference, not part of the default local validation workflow. Do not execute any local `docker` command unless both conditions are true: a non-Docker availability check has already found an existing Docker installation, and the developer or maintainer explicitly requested Docker execution in the current task. A general request to run tests, perform full validation, check Windows behavior, or finish a PR does not satisfy the second condition.

When either condition is absent, do not install or start Docker, pull this image, create the named volumes, launch a container, or use Docker itself to probe availability. Use the repository's Windows CI for durable coverage, or report the exact Windows check that remains unexecuted. If Docker was explicitly requested but the existing installation or daemon is unavailable, stop rather than changing the machine to make it available.

## Pinned environment

Use the image by digest. The older cross-rs 0.2.5 image carries MinGW GCC 7.3 and cannot compile mimalloc 2.3.2.

```sh
IMAGE='ghcr.io/cross-rs/x86_64-pc-windows-gnu@sha256:ac42d6e624b8b63b1803ea960ac901cead36592f9f3b47d22156ccd154c3b83f'

docker pull --platform linux/amd64 "$IMAGE"
docker volume create sugar-path-cross-cargo
docker volume create sugar-path-cross-rustup
docker volume create sugar-path-cross-target

docker run --rm --platform linux/amd64 \
  --entrypoint sh \
  -e HOME=/root \
  -v sugar-path-cross-cargo:/root/.cargo \
  -v sugar-path-cross-rustup:/root/.rustup \
  "$IMAGE" -lc \
  'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.97.0 --target x86_64-pc-windows-gnu'
```

The pinned image used for the historical `9712b6e` checkpoint contains MinGW GCC 13 and Wine 10. Named volumes retain the Rust toolchain and target artifacts.

## Tests and benchmark smoke checks

```sh
docker run --rm --platform linux/amd64 \
  -v sugar-path-cross-cargo:/tmp/home/.cargo \
  -v sugar-path-cross-rustup:/tmp/home/.rustup \
  -v "$PWD":/work:ro \
  -v sugar-path-cross-target:/work/target \
  "$IMAGE" sh -lc \
  'cd /work && /tmp/home/.cargo/bin/cargo test --target x86_64-pc-windows-gnu -p sugar_path --all-features --locked'

docker run --rm --platform linux/amd64 \
  -e RUSTFLAGS='-l advapi32' \
  -v sugar-path-cross-cargo:/tmp/home/.cargo \
  -v sugar-path-cross-rustup:/tmp/home/.rustup \
  -v "$PWD":/work:ro \
  -v sugar-path-cross-target:/work/target \
  "$IMAGE" sh -lc \
  'cd /work && /tmp/home/.cargo/bin/cargo bench --target x86_64-pc-windows-gnu -p sugar_path --all-features --locked --no-run'
```

At the historical `9712b6e` checkpoint, all library, integration, and documentation tests passed, all six benchmark binaries compiled, and the relative suite executed with `--quick`. The extra `advapi32` link flag works around three unresolved Windows-GNU symbols from mimalloc-safe 0.1.64; it is a link-only compatibility flag.

## Historical allocation snapshots

```sh
docker run --rm --platform linux/amd64 \
  -e RUSTFLAGS='-l advapi32' \
  -v sugar-path-cross-cargo:/tmp/home/.cargo \
  -v sugar-path-cross-rustup:/tmp/home/.rustup \
  -v "$PWD":/work:ro \
  -v "$PWD/benchmarks/allocations":/snapshots \
  -v sugar-path-cross-target:/work/target \
  "$IMAGE" sh -lc \
  'cd /work && /tmp/home/.cargo/bin/cargo run --release --target x86_64-pc-windows-gnu -p sugar_path_track_allocations --locked -- --write /snapshots/x86_64-pc-windows-gnu-default.snap'

docker run --rm --platform linux/amd64 \
  -e RUSTFLAGS='-l advapi32' \
  -v sugar-path-cross-cargo:/tmp/home/.cargo \
  -v sugar-path-cross-rustup:/tmp/home/.rustup \
  -v "$PWD":/work:ro \
  -v "$PWD/benchmarks/allocations":/snapshots \
  -v sugar-path-cross-target:/work/target \
  "$IMAGE" sh -lc \
  'cd /work && /tmp/home/.cargo/bin/cargo run --release --target x86_64-pc-windows-gnu -p sugar_path_track_allocations --features rolldown --locked -- --write /snapshots/x86_64-pc-windows-gnu-rolldown.snap'
```

Each historical snapshot scenario was reproduced identically seven times before the file was written. Those GNU snapshots covered ordinary and verbatim UNC normalization, same-share and different-share relative paths, forward and mixed separators, invalid encoding, and the then-current Rolldown pipelines. The old GNU files are no longer checked in because their scenario matrix predates the breaking API; current native Windows evidence uses the MSVC files generated and checked by GitHub Actions.

## Verbatim UNC regression coverage

The historical development checkpoint at `9712b6e` contained and exposed a defect in which `split_windows_root` treated `//?/UNC` as a complete root and returned a relative path across different verbatim UNC shares. Commit [`4b91ad4`](https://github.com/hyf0/sugar_path/commit/4b91ad4b96c6064a8cf348d01b186a91fe68530e) fixes it. The correctness suite now executes same/different server and share cases for ordinary and verbatim UNC, keeps ordinary and verbatim namespaces distinct, and covers drive, verbatim drive, and device namespace roots. Same-root results must rejoin to the normalized target; different roots must return the normalized absolute target.

In the historical Windows-GNU evidence, the corrected different-share verbatim UNC path used three allocations and requested 116 bytes, down from four allocations and 146 bytes at `9712b6e`, in both the default and Rolldown configurations. This is recorded as a correctness consequence, not as a separately tuned performance result or a claim about the accepted #41 MSVC baseline.
