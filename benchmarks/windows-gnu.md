# Optional Windows GNU execution

This document preserves a pinned Docker/Wine environment for reproducing Windows-GNU correctness and allocation behavior from a non-Windows host. It is optional reference material. Native `windows-latest` CI is the default source of executed Windows tests and allocation snapshots, and Wine under amd64 emulation is not a wall-time performance baseline.

No Docker, Wine, image pull, volume creation, or container command in this document is part of ordinary setup, testing, validation, or PR completion.

## Local execution gate

Do not execute any local `docker` command unless both conditions are true:

1. A non-Docker availability check has already found an existing Docker installation.
2. The developer or maintainer explicitly requested Docker execution in the current task.

A general request to test, validate, check Windows behavior, finish a PR, or reproduce all available evidence does not satisfy the second condition. Do not use Docker itself to probe whether Docker is installed. When either condition is absent, skip this path and use native Windows CI or report the exact Windows check that remains unexecuted. If Docker was explicitly requested but the existing installation or daemon is unavailable, stop rather than installing, starting, or reconfiguring it.

## Pinned environment

The following commands are retained only for a task that has passed the gate above. Use the image by digest; the older cross-rs 0.2.5 image carries MinGW GCC 7.3 and cannot compile mimalloc 2.3.2.

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

The pinned image contains MinGW GCC 13 and Wine 10. Named volumes retain the Rust toolchain and target artifacts.

## Tests and benchmark compilation

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

The extra `advapi32` link flag works around unresolved Windows-GNU symbols from mimalloc-safe 0.1.64; it is a link-only compatibility flag.

## Allocation snapshots

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

Each snapshot scenario is reproduced identically seven times before the file is written. Keep GNU snapshots separate from MSVC snapshots; never relabel one target's output as the other.
