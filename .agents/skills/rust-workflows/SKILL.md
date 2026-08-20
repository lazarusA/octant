---
name: rust-workflows
description: >-
  Essential workflows for building, testing, linting, formatting, running with logs,
  and compiling for WebAssembly or desktop in Octant. Use when running checks, tests,
  or build commands.
---

# Rust Workflows for Octant

This skill provides the standard runbooks for checking, building, linting, testing, and debugging Octant.

## 1. Code Quality & Linting

Always run formatting and clippy before submitting changes.

```bash
# Check formatting
cargo fmt --all -- --check

# Auto-format
cargo fmt --all

# Run Clippy with warnings as errors (matching CI)
cargo clippy --all-targets -- -D warnings
```

## 2. Running Tests

Octant has unit tests and integration tests in `tests/`.

```bash
# Run all tests
cargo test

# Run a specific integration test file
cargo test --test volume_animation_tests
cargo test --test zarr_v3_test
cargo test --test date_tests
cargo test --test grid_tests
cargo test --test pyramid_tests

# Run tests with output printed
cargo test -- --nocapture
```

## 3. Running Octant Locally

Run Octant natively on desktop with configurable logging levels:

```bash
# Standard run
cargo run

# Run with detailed Octant & WGPU logs
RUST_LOG=octant=debug,wgpu=warn cargo run

# Run with trace logging for hyperslab / prefetching debugging
RUST_LOG=octant=trace cargo run

# Run in release mode for full GPU & prefetch performance
cargo run --release
```

## 4. WebAssembly Checks

Octant supports WASM builds. When modifying dependencies or async runtimes, ensure WASM target compiles:

```bash
# Check WASM32 compilation
cargo check --target wasm32-unknown-unknown
```

## 5. Pre-Commit Checklist

- [ ] `cargo fmt --all -- --check` passes cleanly.
- [ ] `cargo clippy --all-targets -- -D warnings` has zero warnings.
- [ ] `cargo test` passes all tests.
- [ ] Checked for accidental `.clone()` or unwrap() calls in hot rendering / data paths.
