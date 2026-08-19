use super::*;
use super::canonical_execution::{
    realized_buffer_for_handle, validate_pre_g5b_dynamic_offset_boundary,
    validate_realized_binding_groups, validate_renderer_timestamp_projection,
};
use crate::plugins::gpu::{
    CurrentRenderComputePipelineTerminal, CurrentRenderIndirectBufferTerminal,
    CurrentRenderPipelineBindGroupsTerminal, CurrentRenderTimestampWritesTerminal,
    GpuComputeOperation, GpuDispatchIntent, GpuRealizedBindGroup, GpuRealizedBuffer,
};

impl Renderer {
    /// Temporary pre-G5B physical realization of one execution-complete canonical compute
    /// operation. The operation is the only source of pipeline, binding, dispatch, and timestamp
    /// meaning; renderer state contributes only already-realized opaque backend handles.
    pub(super) fn encode_canonical_compute_operation(
        &mut self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        operation: &GpuComputeOperation,
        prepared: &PreparedPipelinePass,
        gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
        runtime_resources: &FlowRuntimeResources,
    ) -> Result<EncodedPassEvidence> {
        let pipeline_key = prepared.bindings.pipeline_key.clone();
        let realized_pipeline = match &prepared.pipeline {
            PreparedFlowPipeline::Compute(pipeline) => pipeline,
            PreparedFlowPipeline::Render(_) => {
                bail!("canonical compute operation retained a realized render pipeline")
            }
        };
        if realized_pipeline.descriptor() != operation.pipeline() {
            bail!("canonical compute operation pipeline disagrees with its G4C3 realized pipeline");
        }
        if &prepared.bindings.runtime_bindings != operation.bindings() {
            bail!(
                "canonical compute operation bindings disagree with its G4C2 realized binding set"
            );
        }
        validate_pre_g5b_dynamic_offset_boundary("compute", operation.bindings())?;
        validate_realized_binding_groups(
            "compute",
            operation.bindings(),
            &prepared.bindings.bind_groups,
        )?;
        validate_renderer_timestamp_projection(
            "compute",
            operation.timestamp_writes(),
            gpu_timestamp_writes.as_ref(),
        )?;

        let dispatch = match operation.dispatch() {
            GpuDispatchIntent::Direct(size) => CanonicalComputeDispatch::Direct(size.as_array()),
            GpuDispatchIntent::Indirect(access) => CanonicalComputeDispatch::Indirect {
                buffer: realized_buffer_for_handle(
                    "compute",
                    runtime_resources,
                    access.buffer(),
                )?
                .clone(),
                byte_offset: access.range().offset(),
            },
        };
        let dispatch_workgroups = match &dispatch {
            CanonicalComputeDispatch::Direct(size) => Some(*size),
            CanonicalComputeDispatch::Indirect { .. } => None,
        };

        let mut encode_result = Ok(());
        context
            .current_render_execution_bridge()
            .for_compute_pipeline(
                realized_pipeline,
                EncodeCanonicalComputePipeline {
                    context,
                    encoder,
                    bind_groups: &prepared.bindings.bind_groups,
                    dispatch,
                    gpu_timestamp_writes,
                    result: &mut encode_result,
                },
            )?;
        encode_result?;

        Ok(EncodedPassEvidence {
            dispatch_workgroups,
            shader_id: prepared.shader_id.clone(),
            shader_revision: prepared.shader_revision,
            fallback_used: prepared.fallback_used,
            pipeline_key: Some(pipeline_key),
        })
    }
}

enum CanonicalComputeDispatch {
    Direct([u32; 3]),
    Indirect {
        buffer: GpuRealizedBuffer,
        byte_offset: u64,
    },
}

struct EncodeCanonicalComputePipeline<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    bind_groups: &'a [GpuRealizedBindGroup],
    dispatch: CanonicalComputeDispatch,
    gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    result: &'a mut Result<()>,
}

impl CurrentRenderComputePipelineTerminal for EncodeCanonicalComputePipeline<'_> {
    fn use_compute_pipeline(self, pipeline: &ComputePipeline) {
        let operation = CanonicalComputePass {
            context: self.context,
            encoder: self.encoder,
            pipeline,
            bind_groups: self.bind_groups,
            dispatch: self.dispatch,
        };
        if let Some(writes) = self.gpu_timestamp_writes {
            let mut nested_result = Ok(());
            let bridge_result = self
                .context
                .current_render_execution_bridge()
                .for_timestamp_writes(
                    &writes.query_set,
                    EncodeTimestampedCanonicalComputePass {
                        operation,
                        indices: writes.indices,
                        result: &mut nested_result,
                    },
                );
            *self.result = match bridge_result {
                Ok(()) => nested_result,
                Err(error) => Err(error.into()),
            };
        } else {
            *self.result = operation.encode(None);
        }
    }
}

struct CanonicalComputePass<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    pipeline: &'a ComputePipeline,
    bind_groups: &'a [GpuRealizedBindGroup],
    dispatch: CanonicalComputeDispatch,
}

impl CanonicalComputePass<'_> {
    fn encode(self, timestamp: Option<(&QuerySet, GpuPassTimestampIndices)>) -> Result<()> {
        let timestamp_writes = timestamp.map(|(query_set, indices)| ComputePassTimestampWrites {
            query_set,
            beginning_of_pass_write_index: Some(indices.begin),
            end_of_pass_write_index: Some(indices.end),
        });
        let mut pass = self.encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("runengpu_canonical_compute_pass"),
            timestamp_writes,
        });
        pass.set_pipeline(self.pipeline);
        for bind_group in self.bind_groups {
            let index = bind_group.layout_descriptor().group();
            self.context
                .current_render_execution_bridge()
                .for_pipeline_bind_groups(
                    &[bind_group],
                    SetCanonicalComputeBindGroup {
                        pass: &mut pass,
                        index,
                    },
                )?;
        }
        match self.dispatch {
            CanonicalComputeDispatch::Direct([x, y, z]) => pass.dispatch_workgroups(x, y, z),
            CanonicalComputeDispatch::Indirect {
                buffer,
                byte_offset,
            } => self
                .context
                .current_render_execution_bridge()
                .for_indirect_buffer(
                    &buffer,
                    DispatchCanonicalComputeIndirect {
                        pass: &mut pass,
                        byte_offset,
                    },
                )?,
        }
        Ok(())
    }
}

struct EncodeTimestampedCanonicalComputePass<'a> {
    operation: CanonicalComputePass<'a>,
    indices: GpuPassTimestampIndices,
    result: &'a mut Result<()>,
}

impl CurrentRenderTimestampWritesTerminal for EncodeTimestampedCanonicalComputePass<'_> {
    fn write_timestamps(self, query_set: &QuerySet) {
        *self.result = self.operation.encode(Some((query_set, self.indices)));
    }
}

struct SetCanonicalComputeBindGroup<'a, 'pass> {
    pass: &'a mut ComputePass<'pass>,
    index: u32,
}

impl CurrentRenderPipelineBindGroupsTerminal for SetCanonicalComputeBindGroup<'_, '_> {
    fn bind_groups(self, groups: &[&BindGroup]) {
        debug_assert_eq!(groups.len(), 1, "one terminal binds one canonical group");
        self.pass.set_bind_group(self.index, groups[0], &[]);
    }
}

struct DispatchCanonicalComputeIndirect<'a, 'pass> {
    pass: &'a mut ComputePass<'pass>,
    byte_offset: u64,
}

impl CurrentRenderIndirectBufferTerminal for DispatchCanonicalComputeIndirect<'_, '_> {
    fn use_indirect_buffer(self, buffer: &Buffer) {
        self.pass
            .dispatch_workgroups_indirect(buffer, self.byte_offset);
    }
}
