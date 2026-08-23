use super::source;

#[test]
fn renderer_timing_metadata_is_realization_owned() {
    let execute = source("src/plugins/render/renderer/render_flow/execute.rs");
    let realization_start = execute
        .find("fn realize_render_batch<'a>(")
        .expect("renderer realization boundary must remain explicit");
    let raw_execution_start = execute[realization_start..]
        .find("fn execute_realized_batch(")
        .map(|offset| realization_start + offset)
        .expect("temporary raw execution boundary must remain explicit until cutover");
    let realization = &execute[realization_start..raw_execution_start];
    let raw_execution = &execute[raw_execution_start..];

    assert!(
        realization.contains("register_pass_metadata("),
        "renderer timing evidence identity must be fixed during realization"
    );
    assert!(
        !raw_execution.contains("register_pass_metadata("),
        "the temporary raw executor must not create renderer timing evidence identity"
    );
    assert!(
        raw_execution.contains("timestamp_scale_available"),
        "temporary physical timing execution may only consume backend-neutral timestamp scale availability"
    );
}
