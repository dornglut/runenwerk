use engine::plugins::gpu::{
    GpuEntryPointName, GpuExpectedFragmentOutputSignature, GpuExpectedVertexInputSignature,
    GpuFragmentOutputBuiltin, GpuObservedFragmentOutputSignature, GpuObservedVertexInputSignature,
    GpuProgramContractCause, GpuShaderIoLocation, GpuShaderIoScalarClass, GpuShaderIoValueType,
    GpuVertexInputBuiltin, compare_fragment_output_signatures, compare_vertex_input_signatures,
};

fn entry_point(name: &str) -> GpuEntryPointName {
    GpuEntryPointName::new(name).expect("test entry-point name should be valid")
}

fn io_type(scalar_class: GpuShaderIoScalarClass, vector_width: u8) -> GpuShaderIoValueType {
    GpuShaderIoValueType::try_new(scalar_class, vector_width)
        .expect("test shader IO type should be valid")
}

fn location(
    location: u32,
    scalar_class: GpuShaderIoScalarClass,
    vector_width: u8,
) -> GpuShaderIoLocation {
    GpuShaderIoLocation::new(location, io_type(scalar_class, vector_width))
}

#[test]
fn vertex_input_comparison_normalizes_location_order() {
    let entry_point = entry_point("vs_main");
    let expected = GpuExpectedVertexInputSignature::new(
        entry_point.clone(),
        [
            location(1, GpuShaderIoScalarClass::Uint, 1),
            location(0, GpuShaderIoScalarClass::Float, 3),
        ],
    )
    .expect("expected vertex signature should normalize");
    let observed = GpuObservedVertexInputSignature::new(
        entry_point,
        [
            location(0, GpuShaderIoScalarClass::Float, 3),
            location(1, GpuShaderIoScalarClass::Uint, 1),
        ],
        [],
    )
    .expect("observed vertex signature should normalize");

    compare_vertex_input_signatures(&expected, &observed)
        .expect("equivalent normalized signatures should agree");
    assert_eq!(
        expected
            .locations()
            .map(|location| location.location())
            .collect::<Vec<_>>(),
        [0, 1]
    );
}

#[test]
fn fragment_output_comparison_accepts_an_exact_signature() {
    let entry_point = entry_point("fs_main");
    let expected = GpuExpectedFragmentOutputSignature::new(
        entry_point.clone(),
        [location(0, GpuShaderIoScalarClass::Float, 4)],
    )
    .unwrap();
    let observed = GpuObservedFragmentOutputSignature::new(
        entry_point,
        [location(0, GpuShaderIoScalarClass::Float, 4)],
        [GpuFragmentOutputBuiltin::FragDepth],
    )
    .unwrap();

    compare_fragment_output_signatures(&expected, &observed)
        .expect("exact fragment signatures should agree");
}

#[test]
fn stage_io_comparison_rejects_wrong_entry_point() {
    let expected = GpuExpectedVertexInputSignature::new(
        entry_point("vs_main"),
        [location(0, GpuShaderIoScalarClass::Float, 3)],
    )
    .unwrap();
    let observed = GpuObservedVertexInputSignature::new(
        entry_point("other_vs"),
        [location(0, GpuShaderIoScalarClass::Float, 3)],
        [],
    )
    .unwrap();

    let error = compare_vertex_input_signatures(&expected, &observed)
        .expect_err("observations for another entry point must be rejected");
    assert_eq!(
        error.cause(),
        GpuProgramContractCause::PipelineStageIoMismatch
    );
}

#[test]
fn observed_builtins_are_separate_normalized_facts() {
    let observed = GpuObservedVertexInputSignature::new(
        entry_point("vs_main"),
        [],
        [
            GpuVertexInputBuiltin::InstanceIndex,
            GpuVertexInputBuiltin::VertexIndex,
        ],
    )
    .unwrap();

    assert_eq!(
        observed.builtins().collect::<Vec<_>>(),
        [
            GpuVertexInputBuiltin::VertexIndex,
            GpuVertexInputBuiltin::InstanceIndex,
        ]
    );
    assert_eq!(observed.locations().len(), 0);
}

#[test]
fn observed_signatures_reject_duplicate_builtins() {
    let error = GpuObservedFragmentOutputSignature::new(
        entry_point("fs_main"),
        [],
        [
            GpuFragmentOutputBuiltin::SampleMask,
            GpuFragmentOutputBuiltin::SampleMask,
        ],
    )
    .expect_err("duplicate observed builtins must be rejected");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::StageIoSignatureInvalid
    );
}

#[test]
fn stage_io_signatures_reject_duplicate_locations() {
    let error = GpuObservedVertexInputSignature::new(
        entry_point("vs_main"),
        [
            location(2, GpuShaderIoScalarClass::Float, 2),
            location(2, GpuShaderIoScalarClass::Float, 2),
        ],
        [],
    )
    .expect_err("duplicate shader locations must be rejected");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::StageIoSignatureInvalid
    );
}

#[test]
fn stage_io_comparison_rejects_type_mismatch() {
    let entry_point = entry_point("vs_main");
    let expected = GpuExpectedVertexInputSignature::new(
        entry_point.clone(),
        [location(0, GpuShaderIoScalarClass::Float, 3)],
    )
    .unwrap();
    let observed = GpuObservedVertexInputSignature::new(
        entry_point,
        [location(0, GpuShaderIoScalarClass::Float, 4)],
        [],
    )
    .unwrap();

    let error = compare_vertex_input_signatures(&expected, &observed)
        .expect_err("vector-width disagreement must be rejected");
    assert_eq!(
        error.cause(),
        GpuProgramContractCause::PipelineStageIoMismatch
    );
}

#[test]
fn stage_io_comparison_rejects_missing_and_extra_locations() {
    let entry_point = entry_point("fs_main");
    let expected = GpuExpectedFragmentOutputSignature::new(
        entry_point.clone(),
        [location(0, GpuShaderIoScalarClass::Float, 4)],
    )
    .unwrap();
    let empty = GpuObservedFragmentOutputSignature::new(entry_point.clone(), [], []).unwrap();
    let missing = compare_fragment_output_signatures(&expected, &empty)
        .expect_err("a missing observed location must be rejected");
    assert_eq!(
        missing.cause(),
        GpuProgramContractCause::PipelineStageIoMismatch
    );

    let expected_empty = GpuExpectedFragmentOutputSignature::new(entry_point.clone(), []).unwrap();
    let observed = GpuObservedFragmentOutputSignature::new(
        entry_point,
        [location(0, GpuShaderIoScalarClass::Float, 4)],
        [],
    )
    .unwrap();
    let extra = compare_fragment_output_signatures(&expected_empty, &observed)
        .expect_err("an unexpected observed location must be rejected");
    assert_eq!(
        extra.cause(),
        GpuProgramContractCause::PipelineStageIoMismatch
    );
}

#[test]
fn stage_io_value_types_reject_invalid_vector_widths() {
    for vector_width in [0, 5] {
        let error = GpuShaderIoValueType::try_new(GpuShaderIoScalarClass::Float, vector_width)
            .expect_err("shader IO vector width must remain in one through four");
        assert_eq!(
            error.cause(),
            GpuProgramContractCause::StageIoSignatureInvalid
        );
    }
}
