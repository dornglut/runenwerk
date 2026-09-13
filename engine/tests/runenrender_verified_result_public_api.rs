use engine::plugins::render::deterministic_execution::{
    RenderDeterministicResultFormationError, RenderDeterministicVerifiedSubmissionError,
    SubmittedDeterministicRender, submit_deterministic_render_for_verified_result,
};

#[test]
fn verified_result_formation_surface_is_public_to_downstream_consumers() {
    let _ = submit_deterministic_render_for_verified_result;
    let _ = SubmittedDeterministicRender::try_form_verified_result;

    fn assert_public_error<E: std::error::Error + 'static>() {}
    assert_public_error::<RenderDeterministicVerifiedSubmissionError>();
    assert_public_error::<RenderDeterministicResultFormationError>();
}
