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

#[cfg(test)]
mod tests {
    #[test]
    fn ordinary_submission_delegates_to_the_existing_canonical_path() {
        let source = include_str!("ordinary_submission.rs");
        let method = source
            .split_once("    pub async fn submit_work(")
            .expect("ordinary submission method must remain present")
            .1
            .split_once("\n    }\n}")
            .expect("ordinary submission method must remain bounded")
            .0;

        let prepare_graph = method
            .find("GpuPreparedWorkGraph::prepare(label, fragments)?")
            .expect("ordinary work must use canonical graph preparation");
        let prepare_submission = method
            .find("self.prepare_submission(graph).await?")
            .expect("ordinary work must use canonical submission preparation");
        let submit_prepared = method
            .find("self.submit_prepared(prepared)")
            .expect("ordinary work must use canonical prepared submission");

        assert!(prepare_graph < prepare_submission);
        assert!(prepare_submission < submit_prepared);
        for forbidden in [
            "WgpuExecutionState::new",
            "prepare_execution_plan(",
            "encode_submit_and_register(",
            "device.create_command_encoder",
            "queue.submit",
        ] {
            assert!(
                !method.contains(forbidden),
                "ordinary submission must not duplicate execution authority through {forbidden:?}"
            );
        }
    }
}
