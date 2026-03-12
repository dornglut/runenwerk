use anyhow::Result;
use std::sync::Arc;
use wgpu::{
    Adapter, Device, DeviceDescriptor, ExperimentalFeatures, Features, Limits, MemoryHints, Queue,
    Trace,
};

pub async fn request_device_and_queue(adapter: &Adapter) -> Result<(Arc<Device>, Arc<Queue>)> {
    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            label: Some("engine_device"),
            required_features: Features::empty(),
            required_limits: Limits::default(),
            experimental_features: ExperimentalFeatures::disabled(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
        })
        .await?;
    Ok((Arc::new(device), Arc::new(queue)))
}
