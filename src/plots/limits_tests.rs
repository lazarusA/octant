//! The device's storage limits: requested from the adapter, never lowered.

use super::limits::{max_storage_bytes, raised, storage_bytes_of};

const MIB: u64 = 1024 * 1024;

fn limits(binding: u64, buffer: u64) -> wgpu::Limits {
    wgpu::Limits {
        max_storage_buffer_binding_size: binding,
        max_buffer_size: buffer,
        ..wgpu::Limits::default()
    }
}

#[test]
fn the_adapter_raises_the_storage_limits() {
    let asked = raised(
        limits(128 * MIB, 256 * MIB),
        &limits(2048 * MIB, 4096 * MIB),
    );
    assert_eq!(asked.max_storage_buffer_binding_size, 2048 * MIB);
    assert_eq!(asked.max_buffer_size, 4096 * MIB);
}

#[test]
fn a_smaller_adapter_never_lowers_them() {
    let asked = raised(limits(128 * MIB, 256 * MIB), &limits(64 * MIB, 64 * MIB));
    assert_eq!(asked.max_storage_buffer_binding_size, 128 * MIB);
    assert_eq!(asked.max_buffer_size, 256 * MIB);
}

#[test]
fn one_buffer_holds_one_binding_within_the_buffer_size() {
    assert_eq!(
        storage_bytes_of(&limits(2048 * MIB, 1024 * MIB)),
        1024 * MIB
    );
    assert_eq!(storage_bytes_of(&limits(512 * MIB, 4096 * MIB)), 512 * MIB);
}

#[test]
fn without_a_device_the_webgpu_default_applies() {
    // Tests never record a device: 128 MiB, the WebGPU default.
    assert_eq!(max_storage_bytes() as u64, 128 * MIB);
}
