use anyhow::Result;
use std::sync::Arc;
use wgpu::{
    Adapter, Device, DeviceDescriptor, ExperimentalFeatures, Features, Limits, MemoryHints, Queue,
    Trace,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderBackendTimingCapabilities {
    pub timestamp_query: bool,
}

impl RenderBackendTimingCapabilities {
    pub fn from_adapter_features(features: Features) -> Self {
        Self {
            timestamp_query: features.contains(Features::TIMESTAMP_QUERY),
        }
    }
}

pub async fn request_device_and_queue(
    adapter: &Adapter,
) -> Result<(Arc<Device>, Arc<Queue>, RenderBackendTimingCapabilities)> {
    let supported_features = adapter.features();
    let required_features = if supported_features.contains(Features::TIMESTAMP_QUERY) {
        Features::TIMESTAMP_QUERY
    } else {
        Features::empty()
    };
    let timing_capabilities =
        RenderBackendTimingCapabilities::from_adapter_features(required_features);
    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            label: Some("engine_device"),
            required_features,
            required_limits: Limits::default(),
            experimental_features: ExperimentalFeatures::disabled(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
        })
        .await?;
    Ok((Arc::new(device), Arc::new(queue), timing_capabilities))
}
