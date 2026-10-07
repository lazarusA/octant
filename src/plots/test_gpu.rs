//! Test-only GPU device for the plot GPU tests, which skip (and pass) when
//! this returns `None`.

/// A device on the default adapter, requesting the features `select` returns
/// for it; `None` when there is no adapter or `select` rejects it.
pub(crate) fn device_with(
    select: impl FnOnce(&wgpu::Adapter) -> Option<wgpu::Features>,
) -> Option<(wgpu::Device, wgpu::Queue)> {
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(async {
        let adapter = wgpu::Instance::default()
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok()?;
        let required_features = select(&adapter)?;
        let descriptor = wgpu::DeviceDescriptor {
            required_features,
            ..Default::default()
        };
        adapter.request_device(&descriptor).await.ok()
    })
}

/// A device on the default adapter without optional features.
pub(crate) fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    device_with(|_| Some(wgpu::Features::empty()))
}
