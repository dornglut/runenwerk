use engine::plugins::render::{
    AdmittedRender, PreparedRadianceOutput, PreparedRender, RenderAdmissionError,
    RenderCapturedRadiance, RenderExecutionError, RenderRadianceCaptureError,
    RenderRadianceCaptureRequest, RenderRadianceCaptureRequestError, RenderResultFormationError,
    RenderResultSubmissionError, SubmittedRender, SubmittedRenderForResult, admit_render,
    prepare_render, submit_render, submit_render_for_result,
};
use runen_gpu::{GpuContext, GpuReadbackOperation, GpuSubmission};

#[test]
fn ordinary_semantic_renderer_surface_is_public_to_downstream_consumers() {
    let _ = admit_render;
    let _ = prepare_render;
    let _ = submit_render;
    let _ = submit_render_for_result;
    let _ = AdmittedRender::admitted_plan;
    let _ = PreparedRender::admitted_plan;
    let _ = PreparedRender::work_set;
    let _ = PreparedRender::radiance_outputs;
    let _ = PreparedRender::radiance_output;
    let _ = PreparedRadianceOutput::import;
    let _ = SubmittedRender::admitted_plan;
    let _ = SubmittedRender::submission_status;
    let _ = SubmittedRender::object_identity_decoder;
    let _ = SubmittedRenderForResult::admitted_plan;
    let _ = SubmittedRenderForResult::submission_status;
    let _ = SubmittedRenderForResult::object_identity_decoder;
    let _ = SubmittedRenderForResult::try_form_result;
    let _ = SubmittedRenderForResult::request_radiance_capture;
    let _ = SubmittedRenderForResult::capture_radiance;
    let _ = RenderRadianceCaptureRequest::source;
    let _ = RenderRadianceCaptureRequest::readback_id;
    let _ = GpuReadbackOperation::new;
    let _ = RenderAdmissionError::kind;
    let _ = RenderExecutionError::kind;
    let _ = RenderResultSubmissionError::kind;
    let _ = RenderResultFormationError::kind;

    fn assert_prepared_output_surface(output: &PreparedRadianceOutput<'_>) {
        let _ = output.output_index();
        let _ = output.resource();
        let _ = output.texture();
        let _ = output.export_relationship();
    }

    let _ = assert_prepared_output_surface;

    fn assert_capture_surface(
        submitted: &SubmittedRenderForResult,
        request: RenderRadianceCaptureRequest,
        context: &GpuContext,
        product_submission: &GpuSubmission,
    ) -> Result<RenderCapturedRadiance, RenderRadianceCaptureError> {
        submitted.capture_radiance(request, context, product_submission)
    }

    let _ = assert_capture_surface;

    fn assert_public_error<E: std::error::Error + 'static>() {}
    assert_public_error::<RenderAdmissionError>();
    assert_public_error::<RenderExecutionError>();
    assert_public_error::<RenderResultSubmissionError>();
    assert_public_error::<RenderResultFormationError>();
    assert_public_error::<RenderRadianceCaptureRequestError>();
    assert_public_error::<RenderRadianceCaptureError>();
}
