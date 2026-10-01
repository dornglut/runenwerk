use engine::plugins::render::{
    AdmittedRender, PreparedRadianceOutput, PreparedRender, RenderAdmissionError,
    RenderCapturedRadiance, RenderExecutionError, RenderRadianceCaptureError,
    RenderRadianceCaptureRequest, RenderRadianceCaptureRequestError, RenderResultFormationError,
    RenderVerifiedSubmissionError, SubmittedRender, admit_render, prepare_render, submit_render,
    submit_render_for_result,
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
    let _ = SubmittedRender::admitted_plan;
    let _ = SubmittedRender::submission_status;
    let _ = SubmittedRender::try_form_result;
    let _ = SubmittedRender::request_radiance_capture;
    let _ = SubmittedRender::capture_radiance;
    let _ = RenderRadianceCaptureRequest::source;
    let _ = RenderRadianceCaptureRequest::readback_id;
    let _ = GpuReadbackOperation::new;

    fn assert_prepared_output_surface(output: &PreparedRadianceOutput<'_>) {
        let _ = output.output_index();
        let _ = output.resource();
        let _ = output.texture();
        let _ = output.export_relationship();
    }

    let _ = assert_prepared_output_surface;

    fn assert_capture_surface(
        submitted: &SubmittedRender,
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
    assert_public_error::<RenderVerifiedSubmissionError>();
    assert_public_error::<RenderResultFormationError>();
    assert_public_error::<RenderRadianceCaptureRequestError>();
    assert_public_error::<RenderRadianceCaptureError>();
}
