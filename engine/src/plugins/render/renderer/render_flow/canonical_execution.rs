use super::*;
use crate::plugins::gpu::{
    GpuBufferHandle, GpuRealizedBindGroup, GpuRealizedBuffer, GpuRealizedTexture,
    GpuRealizedTextureView, GpuRuntimeBindingResource, GpuRuntimeBindingSet, GpuTextureHandle,
    GpuTextureViewHandle, GpuTimestampWrites,
};

/// Returns the already-realized G4 buffer matching one canonical logical handle.
///
/// Canonical G5A encoding is not allowed to create resources. Phase one must have realized every
/// resource required by execution, and phase two may only borrow those authoritative opaque
/// records through the current lexical execution bridge.
pub(super) fn realized_buffer_for_handle<'a>(
    operation_kind: &str,
    runtime_resources: &'a FlowRuntimeResources,
    handle: &GpuBufferHandle,
) -> Result<&'a GpuRealizedBuffer> {
    runtime_resources
        .buffers
        .values()
        .chain(runtime_resources.invocation_uniform_buffers.values())
        .find(|resource| &resource.handle == handle)
        .map(|resource| &resource.realized)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "canonical {operation_kind} operation references buffer '{}' without an already-realized G4 resource",
                handle.descriptor().common().label().as_str()
            )
        })
}

/// Returns the already-realized G4 texture matching one canonical logical handle.
pub(super) fn realized_texture_for_handle<'a>(
    operation_kind: &str,
    runtime_resources: &'a FlowRuntimeResources,
    handle: &GpuTextureHandle,
) -> Result<&'a GpuRealizedTexture> {
    runtime_resources
        .textures
        .values()
        .chain(runtime_resources.invocation_history_textures.values())
        .find(|resource| &resource.handle == handle)
        .map(|resource| &resource.realized)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "canonical {operation_kind} operation references texture '{}' without an already-realized G4 resource",
                handle.descriptor().common().label().as_str()
            )
        })
}

/// Returns the already-realized G4 texture view matching one canonical logical handle.
pub(super) fn realized_texture_view_for_handle<'a>(
    operation_kind: &str,
    runtime_resources: &'a FlowRuntimeResources,
    handle: &GpuTextureViewHandle,
) -> Result<&'a GpuRealizedTextureView> {
    runtime_resources
        .textures
        .values()
        .chain(runtime_resources.invocation_history_textures.values())
        .find(|resource| &resource.view_handle == handle)
        .map(|resource| &resource.realized_view)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "canonical {operation_kind} operation references texture view '{}' without an already-realized G4 resource",
                handle.descriptor().common().label().as_str()
            )
        })
}

/// The accepted G5A contract owns logical `u64` dynamic offsets, while G5B owns ordered backend
/// offset slices and checked narrowing. The temporary renderer execution bridge must therefore fail
/// closed instead of discarding logical dynamic-offset meaning.
pub(super) fn validate_pre_g5b_dynamic_offset_boundary(
    operation_kind: &str,
    bindings: &GpuRuntimeBindingSet,
) -> Result<()> {
    let has_dynamic_offset = bindings.values().any(|value| {
        value.resources().any(|resource| {
            matches!(
                resource,
                GpuRuntimeBindingResource::Buffer(binding) if binding.dynamic_offset().is_some()
            )
        })
    });
    if has_dynamic_offset {
        bail!(
            "canonical {operation_kind} operation requires dynamic-offset lowering owned by G5B; the temporary renderer execution bridge cannot discard logical dynamic offsets"
        );
    }
    Ok(())
}

/// G4C2 may contribute only the opaque physical bind groups corresponding to the canonical logical
/// binding set. Count and layout are rechecked at the lexical execution seam so renderer-owned
/// realization cannot become a second binding authority.
pub(super) fn validate_realized_binding_groups(
    operation_kind: &str,
    bindings: &GpuRuntimeBindingSet,
    realized: &[GpuRealizedBindGroup],
) -> Result<()> {
    let logical = bindings.groups();
    if logical.len() != realized.len() {
        bail!(
            "canonical {operation_kind} operation has {} logical binding groups but {} G4C2 realized groups",
            logical.len(),
            realized.len()
        );
    }
    for (logical, realized) in logical.iter().zip(realized) {
        if logical.layout() != realized.layout_descriptor() {
            bail!(
                "canonical {operation_kind} binding group {} disagrees with its G4C2 realized layout",
                logical.layout().group()
            );
        }
    }
    Ok(())
}

/// The current renderer timing bridge physically realizes the renderer's existing begin/end pair.
/// The logical `GpuTimestampWrites` value remains the authority for query-set identity and semantic
/// placement; this seam only verifies that the retained physical sidecar is the same pair.
pub(super) fn validate_renderer_timestamp_projection(
    operation_kind: &str,
    logical: Option<&GpuTimestampWrites>,
    physical: Option<&GpuPassTimestampWrites>,
) -> Result<()> {
    match (logical, physical) {
        (None, None) => Ok(()),
        (Some(logical), Some(physical)) => {
            if physical.query_set.logical_identity() != logical.query_set().diagnostic_identity()
                || logical.beginning_of_pass() != Some(physical.indices.begin)
                || logical.end_of_pass() != Some(physical.indices.end)
            {
                bail!(
                    "canonical {operation_kind} timestamp sidecar disagrees with the explicit logical begin/end timestamp state"
                );
            }
            Ok(())
        }
        (None, Some(_)) => bail!(
            "canonical {operation_kind} operation has no logical timestamps but retained a physical timestamp sidecar"
        ),
        (Some(_), None) => bail!(
            "canonical {operation_kind} operation requires logical timestamps but has no physical timestamp realization"
        ),
    }
}
