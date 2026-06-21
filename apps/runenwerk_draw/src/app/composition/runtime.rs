//! Atomic Draw composition state and extension runtime.

use ui_composition::{
    AppProfileId, AppSchemaVersion, CompositionBundleCandidate, CompositionCompatibility,
    CompositionCompatibilityRequirement, CompositionDefinitionV1, CompositionState,
};

use super::{
    DRAWING_APP_PROFILE, DrawingCompositionDiagnosticCode as Code,
    DrawingCompositionDiagnosticRecord as Record, DrawingCompositionDiagnosticStage as Stage,
    DrawingCompositionDiagnosticSubject as Subject, DrawingCompositionExtensionV1,
    DrawingCompositionRejection, builtin_drawing_composition_definition,
    builtin_drawing_composition_extension,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingCompositionRuntime {
    composition: CompositionState,
    extension: DrawingCompositionExtensionV1,
}

impl DrawingCompositionRuntime {
    pub fn new(
        definition: CompositionDefinitionV1,
        extension: DrawingCompositionExtensionV1,
    ) -> Result<Self, DrawingCompositionRejection> {
        let composition = CompositionState::form(definition).map_err(|rejection| {
            DrawingCompositionRejection::single(
                Record::error(
                    Code::DefinitionInvalid,
                    Stage::Runtime,
                    Subject::General("draw_composition_definition".to_owned()),
                    "Form a valid Draw composition definition before installing runtime state.",
                )
                .with_context(
                    "formation_diagnostics",
                    rejection.diagnostics().len().to_string(),
                ),
            )
        })?;
        extension.validate_against(&composition)?;
        Ok(Self {
            composition,
            extension,
        })
    }

    pub fn builtin() -> Result<Self, DrawingCompositionRejection> {
        let definition = builtin_drawing_composition_definition()?;
        let composition = CompositionState::form(definition.clone()).map_err(|rejection| {
            DrawingCompositionRejection::single(
                Record::error(
                    Code::DefinitionInvalid,
                    Stage::Runtime,
                    Subject::Layout(definition.id()),
                    "Fix the built-in Draw composition definition before starting the app.",
                )
                .with_context(
                    "formation_diagnostics",
                    rejection.diagnostics().len().to_string(),
                ),
            )
        })?;
        let extension = builtin_drawing_composition_extension(&composition)?;
        extension.validate_against(&composition)?;
        Ok(Self {
            composition,
            extension,
        })
    }

    pub fn composition(&self) -> &CompositionState {
        &self.composition
    }

    pub fn extension(&self) -> &DrawingCompositionExtensionV1 {
        &self.extension
    }

    pub fn linked_bundle_candidate(
        &self,
    ) -> Result<CompositionBundleCandidate, DrawingCompositionRejection> {
        let app_profile = AppProfileId::new(DRAWING_APP_PROFILE)
            .map_err(|error| linked_bundle_error("Use the compiled-in Draw app profile.", error))?;
        let schema = AppSchemaVersion::new(1)
            .map_err(|error| linked_bundle_error("Use Draw app schema version 1.", error))?;
        let compatibility = CompositionCompatibility::new(app_profile.clone(), schema, schema)
            .map_err(|error| {
                linked_bundle_error("Use a valid Draw app compatibility range.", error)
            })?;
        let requirement = CompositionCompatibilityRequirement {
            app_profile,
            app_schema_version: schema,
        };
        let candidate = CompositionBundleCandidate::form(
            self.composition.definition().clone(),
            compatibility,
            vec![self.extension.canonical_payload()?],
        )
        .map_err(|error| linked_bundle_error("Form a linked Draw composition bundle.", error))?;
        candidate.validate(Some(&requirement)).map_err(|error| {
            linked_bundle_error("Validate the complete linked Draw bundle.", error)
        })?;
        Ok(candidate)
    }
}

fn linked_bundle_error(
    message: &'static str,
    error: impl std::fmt::Display,
) -> DrawingCompositionRejection {
    DrawingCompositionRejection::single(
        Record::error(
            Code::LinkedBundleInvalid,
            Stage::Runtime,
            Subject::General("draw_composition_bundle".to_owned()),
            message,
        )
        .with_context("source_error", error.to_string()),
    )
}
