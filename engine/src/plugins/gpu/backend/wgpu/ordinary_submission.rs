use crate::plugins::gpu::{
    GpuContext, GpuPreparedWorkGraph, GpuResourceLabel, GpuSubmission, GpuWorkFragment,
    GpuWorkSubmissionError,
};

impl GpuContext {
    /// Prepares, validates, and submits ordinary authored GPU work through the
    /// same canonical authorities as the explicit advanced path.
    pub async fn submit_work(
        &self,
        label: GpuResourceLabel,
        fragments: impl IntoIterator<Item = GpuWorkFragment>,
    ) -> Result<GpuSubmission, GpuWorkSubmissionError> {
        let graph = GpuPreparedWorkGraph::prepare(label, fragments)?;
        let prepared = self.prepare_submission(graph).await?;
        self.submit_prepared(prepared).map_err(|rejected| {
            let (_, reason) = rejected.into_parts();
            GpuWorkSubmissionError::SubmissionRejected(reason)
        })
    }
}
