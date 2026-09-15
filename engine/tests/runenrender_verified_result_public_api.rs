use engine::plugins::render::deterministic_execution::{
    PreparedDeterministicRadianceOutput, PreparedDeterministicRender,
    RenderCapturedDeterministicRadiance, RenderDeterministicRadianceCaptureError,
    RenderDeterministicRadianceCaptureRequest, RenderDeterministicRadianceCaptureRequestError,
    RenderDeterministicResultFormationError, RenderDeterministicVerifiedSubmissionError,
    SubmittedDeterministicRender, prepare_deterministic_render,
    submit_deterministic_render_for_verified_result,
};
use runen_gpu::{GpuContext, GpuReadbackOperation, GpuSubmission};

#[test]
fn verified_result_formation_surface_is_public_to_downstream_consumers() {
    let _ = prepare_deterministic_render;
    let _ = PreparedDeterministicRender::admitted;
    let _ = PreparedDeterministicRender::work_set;
    let _ = PreparedDeterministicRender::radiance_outputs;
    let _ = PreparedDeterministicRender::radiance_output;
    let _ = PreparedDeterministicRadianceOutput::output_index;
    let _ = PreparedDeterministicRadianceOutput::resource;
    let _ = PreparedDeterministicRadianceOutput::texture;
    let _ = PreparedDeterministicRadianceOutput::export_relationship;
    let _ = PreparedDeterministicRadianceOutput::import;
    let _ = submit_deterministic_render_for_verified_result;
    let _ = SubmittedDeterministicRender::submission_status;
    let _ = SubmittedDeterministicRender::try_form_verified_result;
    let _ = SubmittedDeterministicRender::request_deterministic_radiance_capture;
    let _ = SubmittedDeterministicRender::capture_deterministic_radiance;
    let _ = RenderDeterministicRadianceCaptureRequest::source;
    let _ = RenderDeterministicRadianceCaptureRequest::readback_id;
    let _ = GpuReadbackOperation::new;

    fn assert_capture_surface(
        submitted: &SubmittedDeterministicRender,
        request: RenderDeterministicRadianceCaptureRequest,
        context: &GpuContext,
        product_submission: &GpuSubmission,
    ) -> Result<RenderCapturedDeterministicRadiance, RenderDeterministicRadianceCaptureError> {
        submitted.capture_deterministic_radiance(request, context, product_submission)
    }

    let _ = assert_capture_surface;

    fn assert_public_error<E: std::error::Error + 'static>() {}
    assert_public_error::<RenderDeterministicVerifiedSubmissionError>();
    assert_public_error::<RenderDeterministicResultFormationError>();
    assert_public_error::<RenderDeterministicRadianceCaptureRequestError>();
    assert_public_error::<RenderDeterministicRadianceCaptureError>();
}
