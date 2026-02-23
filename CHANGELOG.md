# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.3.0](https://github.com/hyf0/sugar_path/compare/v1.2.1...v1.3.0) - 2026-02-23

### Added

- always remove last `/`, refactor tests with str-based equality check ([#29](https://github.com/hyf0/sugar_path/pull/29))

### Fixed

- ensure consistent normalization of UNC paths with trailing separators and `\\` ([#30](https://github.com/hyf0/sugar_path/pull/30))

### Other

- optimize `absolutize` to avoid allocation for already-absolute paths ([#34](https://github.com/hyf0/sugar_path/pull/34))
- add benchmarks for absolutize on clean absolute, dirty absolute, and relative paths ([#33](https://github.com/hyf0/sugar_path/pull/33))
- optimize normalize() to avoid allocation for already-clean paths ([#32](https://github.com/hyf0/sugar_path/pull/32))
- add normalization benchmarks with additional workload scenarios ([#31](https://github.com/hyf0/sugar_path/pull/31))
- memchr-accelerated fast path for `relative()` ([#27](https://github.com/hyf0/sugar_path/pull/27))

## [1.2.1](https://github.com/hyf0/sugar_path/compare/v1.2.0...v1.2.1) - 2025-10-21

### Other

- reduce memory allocation ([#26](https://github.com/hyf0/sugar_path/pull/26))
- add deep paths fixtures ([#25](https://github.com/hyf0/sugar_path/pull/25))
- add missing tests and benchmarks for functions used in rolldown ([#24](https://github.com/hyf0/sugar_path/pull/24))
- upgrade all infra ([#21](https://github.com/hyf0/sugar_path/pull/21))
