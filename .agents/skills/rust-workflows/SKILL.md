---
name: rust-workflows
description: >-
  Essential workflows for building, testing, linting, formatting, running with logs,
  and compiling for WebAssembly or desktop in Octant. Use when running checks, tests,
  or build commands.
---

# Rust Workflows for Octant

This skill provides the standard runbooks for checking, building, linting, testing, and debugging Octant.

**Reference**: `build.md` (the release profile, `wasm-opt` flags, the CI's wasm size budgets, third-party licenses and packaging).

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
# Run all unit and integration tests
cargo test --tests

# Run a specific integration test file
cargo test --test volume_animation_tests
cargo test --test zarr_v3_test
cargo test --test date_tests
cargo test --test grid_tests
cargo test --test pyramid_tests

# Run tests with output printed
cargo test -- --nocapture
```

*Note*: For C-library/FFI test suites (e.g. NetCDF), tests serialize file creation with static mutex test locks to prevent concurrent non-reentrant IO collisions.

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

## 4. WebAssembly & Web Builds

Octant supports browser execution via WASM and WebGPU/WebGL2:

The wasm build compiles C zstd (`zstd-sys`, through `async-tiff`, `icechunk-format` and
`zstd`) and needs a C compiler that targets `wasm32-unknown-unknown`. Linux clang does
(CI); Apple clang does not (since Command Line Tools 26.6: "No available targets are
compatible with triple"). On macOS, unpack LLVM's own clang (native arm64, no Homebrew or
Rosetta) and point the wasm build at it; desktop builds keep Apple clang:

```bash
V=23.1.5; N=LLVM-$V-macOS-ARM64
curl -fLO https://github.com/llvm/llvm-project/releases/download/llvmorg-$V/$N.tar.xz
mkdir -p ~/.local/llvm-wasm
tar -xJf $N.tar.xz -C ~/.local/llvm-wasm --strip-components=1 \
  $N/bin/clang $N/bin/clang-${V%%.*} $N/bin/llvm-ar $N/lib/clang/${V%%.*}/include
# In ~/.zshrc:
export CC_wasm32_unknown_unknown="$HOME/.local/llvm-wasm/bin/clang"
export AR_wasm32_unknown_unknown="$HOME/.local/llvm-wasm/bin/llvm-ar"
```

```bash
# Check WASM32 compilation
cargo check --target wasm32-unknown-unknown

# Build for Web using Trunk
trunk build
trunk build --release

# Validate brotli-compressed WASM bundle size against .github/wasm-size-budget
scripts/check_wasm_size.sh
```

## 5. Pre-Commit Checklist

- [ ] `cargo fmt --all -- --check` passes cleanly.
- [ ] `cargo clippy --all-targets -- -D warnings` has zero warnings.
- [ ] `cargo test --tests` passes all tests.
- [ ] Checked for accidental `.clone()` or unwrap() calls in hot rendering / data paths.
