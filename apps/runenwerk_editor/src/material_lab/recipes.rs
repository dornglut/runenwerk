use asset::{
    AssetDiagnosticCode, AssetDiagnosticRecord, AssetKind, AssetSourceDescriptor, ImportSettings,
};
use material_graph::{MaterialNodeCatalog, MaterialOutputTarget};

use crate::asset_pipeline::EditorImportRecipe;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialLoweringRecipe {
    PreviewMaterial,
    RenderMaterial,
}

impl MaterialLoweringRecipe {
    pub fn from_serialized_target(target: &str) -> Result<Self, AssetDiagnosticRecord> {
        match target {
            "preview" => Ok(Self::PreviewMaterial),
            "render_material" => Ok(Self::RenderMaterial),
            "" => Err(import_profile_diagnostic(
                "material graph import profile has an empty lowering target",
            )),
            other => Err(import_profile_diagnostic(format!(
                "unknown material graph lowering target {other:?}; expected \"preview\" or \"render_material\""
            ))),
        }
    }

    pub const fn output_target(self) -> MaterialOutputTarget {
        match self {
            Self::PreviewMaterial => MaterialOutputTarget::PbrPreview,
            Self::RenderMaterial => MaterialOutputTarget::RenderMaterial,
        }
    }

    pub const fn cache_key_component(self) -> &'static str {
        match self {
            Self::PreviewMaterial => "material-lowering-recipe-v1=preview",
            Self::RenderMaterial => "material-lowering-recipe-v1=render_material",
        }
    }

    pub const fn specialization_policy(self) -> MaterialSpecializationPolicy {
        MaterialSpecializationPolicy::FirstSlice
    }

    pub const fn renderer_parameter_profile(self) -> MaterialRendererParameterProfile {
        match self {
            Self::PreviewMaterial => MaterialRendererParameterProfile::PbrPreview,
            Self::RenderMaterial => MaterialRendererParameterProfile::RenderMaterial,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialSpecializationPolicy {
    FirstSlice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialRendererParameterProfile {
    PbrPreview,
    RenderMaterial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMaterialLoweringRecipe {
    pub recipe: MaterialLoweringRecipe,
    pub output_target: MaterialOutputTarget,
    pub expected_artifact_kind: AssetKind,
    pub cache_key_component: &'static str,
    pub specialization_policy: MaterialSpecializationPolicy,
    pub renderer_parameter_profile: MaterialRendererParameterProfile,
}

impl ResolvedMaterialLoweringRecipe {
    pub fn resolve(
        source: &AssetSourceDescriptor,
        import_recipe: &EditorImportRecipe,
    ) -> Result<Self, Vec<AssetDiagnosticRecord>> {
        let mut diagnostics = Vec::new();
        if source.kind != AssetKind::MaterialGraph
            || import_recipe.key.asset_kind != AssetKind::MaterialGraph
        {
            diagnostics.push(import_profile_diagnostic(format!(
                "material lowering recipe requires a material graph source, got source {:?} and recipe {:?}",
                source.kind, import_recipe.key.asset_kind
            )));
        }
        if import_recipe.expected_artifact_kind != AssetKind::Material {
            diagnostics.push(import_profile_diagnostic(format!(
                "material lowering recipe must publish material artifacts, got {:?}",
                import_recipe.expected_artifact_kind
            )));
        }

        let lowering_target = match &import_recipe.settings {
            ImportSettings::MaterialGraph { lowering_target } => Some(lowering_target.as_str()),
            other => {
                diagnostics.push(import_profile_diagnostic(format!(
                    "material lowering recipe requires material_graph import settings, got {}",
                    other.stable_kind_label()
                )));
                None
            }
        };
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        let lowering_target =
            lowering_target.expect("lowering target exists when diagnostics are empty");
        let recipe = match MaterialLoweringRecipe::from_serialized_target(lowering_target) {
            Ok(recipe) => recipe,
            Err(diagnostic) => return Err(vec![diagnostic]),
        };

        Ok(Self {
            recipe,
            output_target: recipe.output_target(),
            expected_artifact_kind: AssetKind::Material,
            cache_key_component: recipe.cache_key_component(),
            specialization_policy: recipe.specialization_policy(),
            renderer_parameter_profile: recipe.renderer_parameter_profile(),
        })
    }

    pub fn node_catalog(&self) -> MaterialNodeCatalog {
        match self.specialization_policy {
            MaterialSpecializationPolicy::FirstSlice => MaterialNodeCatalog::first_slice(),
        }
    }

    pub fn validate_document_output_target(
        &self,
        document_output_target: MaterialOutputTarget,
    ) -> Result<(), AssetDiagnosticRecord> {
        if document_output_target == self.output_target {
            return Ok(());
        }
        Err(import_profile_diagnostic(format!(
            "material graph document output target {:?} does not match resolved recipe output target {:?}",
            document_output_target, self.output_target
        )))
    }
}

fn import_profile_diagnostic(message: impl Into<String>) -> AssetDiagnosticRecord {
    AssetDiagnosticRecord::error(AssetDiagnosticCode::ImportProfileRejected, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{asset_id, asset_source_id};

    fn source(kind: AssetKind) -> AssetSourceDescriptor {
        AssetSourceDescriptor::new(asset_source_id(1), asset_id(1), kind, "assets/material.ron")
    }

    fn recipe(
        lowering_target: impl Into<String>,
        expected_artifact_kind: AssetKind,
    ) -> EditorImportRecipe {
        EditorImportRecipe::new(
            AssetKind::MaterialGraph,
            "default",
            ImportSettings::MaterialGraph {
                lowering_target: lowering_target.into(),
            },
            expected_artifact_kind,
        )
    }

    #[test]
    fn material_recipe_resolution_accepts_exact_preview_and_render_targets() {
        let preview = ResolvedMaterialLoweringRecipe::resolve(
            &source(AssetKind::MaterialGraph),
            &recipe("preview", AssetKind::Material),
        )
        .expect("preview recipe should resolve");
        let render = ResolvedMaterialLoweringRecipe::resolve(
            &source(AssetKind::MaterialGraph),
            &recipe("render_material", AssetKind::Material),
        )
        .expect("render recipe should resolve");

        assert_eq!(preview.recipe, MaterialLoweringRecipe::PreviewMaterial);
        assert_eq!(preview.output_target, MaterialOutputTarget::PbrPreview);
        assert_eq!(render.recipe, MaterialLoweringRecipe::RenderMaterial);
        assert_eq!(render.output_target, MaterialOutputTarget::RenderMaterial);
        assert_ne!(preview.cache_key_component, render.cache_key_component);
    }

    #[test]
    fn material_recipe_resolution_rejects_empty_unknown_and_incompatible_targets() {
        for bad_target in ["", "Preview", "render", "render_material "] {
            let diagnostics = ResolvedMaterialLoweringRecipe::resolve(
                &source(AssetKind::MaterialGraph),
                &recipe(bad_target, AssetKind::Material),
            )
            .expect_err("bad lowering target should be rejected");
            assert_eq!(
                diagnostics[0].code,
                AssetDiagnosticCode::ImportProfileRejected
            );
        }

        let diagnostics = ResolvedMaterialLoweringRecipe::resolve(
            &source(AssetKind::Texture2D),
            &recipe("preview", AssetKind::Material),
        )
        .expect_err("non-material graph source should be rejected");
        assert_eq!(
            diagnostics[0].code,
            AssetDiagnosticCode::ImportProfileRejected
        );

        let diagnostics = ResolvedMaterialLoweringRecipe::resolve(
            &source(AssetKind::MaterialGraph),
            &recipe("preview", AssetKind::Texture2D),
        )
        .expect_err("non-material artifact target should be rejected");
        assert_eq!(
            diagnostics[0].code,
            AssetDiagnosticCode::ImportProfileRejected
        );
    }

    #[test]
    fn material_recipe_rejects_document_target_mismatch() {
        let preview = ResolvedMaterialLoweringRecipe::resolve(
            &source(AssetKind::MaterialGraph),
            &recipe("preview", AssetKind::Material),
        )
        .expect("preview recipe should resolve");

        let diagnostic = preview
            .validate_document_output_target(MaterialOutputTarget::RenderMaterial)
            .expect_err("document target mismatch should be rejected");

        assert_eq!(diagnostic.code, AssetDiagnosticCode::ImportProfileRejected);
    }
}
