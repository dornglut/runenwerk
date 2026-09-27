use super::*;
use ui_schema::UiSchemaValue;

#[test]
fn architecture_fixtures_compile_evaluate_assert_and_reproduce() {
    let fixture = UiArchitectureFixture::minimal("minimal-label");
    let report = fixture.headless.compile_report();
    let run = fixture.run();

    assert!(report.passed(), "{report:#?}");
    assert_eq!(fixture.fixture_id, "minimal-label");
    assert_eq!(run.artifact.manifest.program_id, "fixture.headless");
    assert_eq!(run.artifact.tables.visual.rows.len(), 1);
    assert_eq!(
        run.artifact.tables.visual.rows[0]
            .operator
            .operator_id
            .as_str(),
        "visual.fixture.title"
    );
    assert_eq!(
        run.output
            .state
            .rows
            .iter()
            .find(|row| row.state_key.as_str() == "state.fixture.title")
            .map(|row| row.revision),
        Some(1)
    );
    assert_eq!(
        run.state.value("state.fixture.title"),
        Some(&UiSchemaValue::string("Inspector"))
    );
    assert_eq!(run.runenui_publication, Ok(()));
    assert_eq!(
        crate::headless_fixture::prove_runenui_minimal_label(&run.artifact, &run.state),
        Ok(())
    );
    assert!(
        run.source_map_assertion
            .assert_artifact(&run.artifact)
            .is_ok()
    );
    assert!(
        run.diagnostic_assertion
            .assert_artifact(&run.artifact)
            .is_ok()
    );
    assert!(run.reproducibility_assertion.passed());
    assert!(run.passed());
}

#[test]
fn runenui_minimal_label_projection_fails_closed_without_resolved_state() {
    let fixture = UiArchitectureFixture::minimal("minimal-label");
    let artifact = fixture.headless.compile();

    assert_eq!(
        crate::headless_fixture::prove_runenui_minimal_label(
            &artifact,
            &ui_state::UiStateModel::default(),
        ),
        Err(crate::headless_fixture::RunenUiHeadlessProofError::MissingTextState)
    );
}

#[test]
fn runenui_minimal_label_projection_fails_closed_on_unexpected_control_shape() {
    let fixture = UiArchitectureFixture::minimal("minimal-label");
    let mut artifact = fixture.headless.compile();
    let duplicate = artifact.tables.controls.rows[0].clone();
    artifact.tables.controls.rows.push(duplicate);

    assert_eq!(
        crate::headless_fixture::prove_runenui_minimal_label(
            &artifact,
            &ui_state::UiStateModel::default(),
        ),
        Err(crate::headless_fixture::RunenUiHeadlessProofError::UnexpectedControlShape)
    );
}
