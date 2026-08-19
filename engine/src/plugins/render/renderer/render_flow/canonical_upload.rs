use super::*;
use crate::plugins::gpu::{
    CurrentRenderBufferUploadTerminal, GpuRealizedBuffer, GpuTransferRegion, GpuUploadOperation,
};

impl Renderer {
    /// Temporary pre-G5B physical realization of one execution-complete canonical Upload.
    ///
    /// `operation` is the only source of destination coverage and payload meaning. The renderer
    /// contributes only the opaque G4C1 buffer realization and the current queue loan. Durable
    /// staging/submission coupling remains G5B authority.
    pub(super) fn encode_canonical_upload_operation(
        &self,
        context: &GpuContext,
        queue: &Queue,
        operation: &GpuUploadOperation,
        realized: &GpuRealizedBuffer,
    ) -> Result<()> {
        let GpuTransferRegion::Buffer(destination) = operation.destination() else {
            bail!(
                "canonical renderer Upload requires texture staging/lowering owned by G5B; the temporary pre-G5B adapter only realizes buffer destinations"
            );
        };
        if realized.logical_identity() != destination.buffer().diagnostic_identity() {
            bail!(
                "canonical Upload destination '{}' disagrees with its G4C1 realized buffer '{}'",
                destination.buffer().diagnostic_identity(),
                realized.logical_identity()
            );
        }

        context
            .current_render_execution_bridge()
            .for_buffer_upload(
                realized,
                WriteCanonicalBufferUpload {
                    queue,
                    byte_offset: destination.range().offset(),
                    contents: operation.payload().as_bytes(),
                },
            )?;
        Ok(())
    }
}

struct WriteCanonicalBufferUpload<'a> {
    queue: &'a Queue,
    byte_offset: u64,
    contents: &'a [u8],
}

impl CurrentRenderBufferUploadTerminal for WriteCanonicalBufferUpload<'_> {
    fn upload_buffer(self, buffer: &Buffer) {
        self.queue
            .write_buffer(buffer, self.byte_offset, self.contents);
    }
}
