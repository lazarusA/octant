# Rust & Octant Guidelines

## Core Principles

1. **Ownership & Borrowing**:
   - Prefer borrowing (`&T`, `&str`, `&[T]`) over `.clone()`.
   - Never clone collections or large structs solely to pass them to read-only functions.
   - Use `Arc<T>` only for shared data across threads; prefer references within local scopes.

2. **Error Handling**:
   - Never use `unwrap()` in production code paths (`err-no-unwrap-prod`). Use `?`, `match`, `if let`, or `expect()` with a detailed invariant message.
   - Return descriptive errors with error chains preserved.

3. **Performance & Allocation**:
   - Pre-allocate collections with `Vec::with_capacity` when the size is known.
   - Avoid `format!()` in hot loops or render passes; use `write!()` or string literals.
   - Zero-copy tensor slicing: avoid unnecessary copies when slicing N-dimensional arrays in `OctantBlock`.

4. **WGPU & Shaders**:
   - Maintain 16-byte std140/std430 alignment on uniform structs with `#[repr(C)]` and `bytemuck::Pod`/`Zeroable`.
   - Always run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all -- --check`.

5. **Antigravity Skills**:
   - Access comprehensive Rust best practices and 265 specialized rules via the `rust-skills` skill.
   - Access Octant architecture and domain runbooks via `rust-workflows`, `octant-data-engine`, `octant-rendering-wgpu`, and `octant-ui-egui`.
