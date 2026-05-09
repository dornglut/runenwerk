use std::collections::BTreeSet;

use asset::{
    ArtifactPayloadKind, ArtifactValidity, AssetArtifactDescriptor, AssetCatalog,
    AssetDiagnosticRecord, AssetId, AssetKind,
};
use editor_preview::{
    ReloadDecision, ReloadStatus, ReloadSubject, RuntimeProductKind, RuntimeProductRef,
};
use texture::{TexturePreviewDescriptor, TextureProductId};
use world_sdf::{FieldProductDescriptor, FieldProductFreshness};

#[derive(Debug, Clone, Default)]
pub struct AssetCatalogRuntime {
    catalog: AssetCatalog,
    diagnostics: Vec<AssetDiagnosticRecord>,
    dirty_assets: BTreeSet<AssetId>,
    reload_statuses: Vec<ReloadStatus>,
    selected_asset_id: Option<AssetId>,
    selected_field_product: Option<FieldProductDescriptor>,
}

impl AssetCatalogRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn catalog(&self) -> &AssetCatalog {
        &self.catalog
    }

    pub fn catalog_mut(&mut self) -> &mut AssetCatalog {
        &mut self.catalog
    }

    pub fn replace_catalog(&mut self, catalog: AssetCatalog) {
        self.catalog = catalog;
        self.dirty_assets.clear();
    }

    pub fn diagnostics(&self) -> &[AssetDiagnosticRecord] {
        &self.diagnostics
    }

    pub fn record_diagnostic(&mut self, diagnostic: AssetDiagnosticRecord) {
        self.diagnostics.push(diagnostic);
    }

    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    pub fn mark_asset_dirty(&mut self, asset_id: AssetId) {
        self.dirty_assets.insert(asset_id);
    }

    pub fn dirty_assets(&self) -> impl Iterator<Item = AssetId> + '_ {
        self.dirty_assets.iter().copied()
    }

    pub fn classify_dirty_assets_for_reload(&mut self) -> Vec<ReloadStatus> {
        let statuses = self
            .dirty_assets
            .iter()
            .copied()
            .map(|asset_id| self.classify_asset_reload(asset_id))
            .collect::<Vec<_>>();
        self.reload_statuses.extend(statuses.iter().cloned());
        statuses
    }

    pub fn classify_asset_reload(&self, asset_id: AssetId) -> ReloadStatus {
        let Some(record) = self.catalog.asset(asset_id) else {
            return ReloadStatus::new(
                ReloadSubject::new(editor_preview::ReloadSubjectKind::Asset, "missing asset"),
                ReloadDecision::Rejected,
                "asset is not present in the catalog",
            );
        };
        let subject = ReloadSubject::asset(record.asset_id, record.kind, record.revision_id);
        let Some(artifact) = record
            .artifact_ids
            .iter()
            .filter_map(|artifact_id| self.catalog.artifact(*artifact_id))
            .next()
        else {
            return ReloadStatus::new(
                subject,
                reload_decision_for_kind(record.kind),
                "asset has no runtime artifact yet",
            );
        };
        self.classify_artifact_reload(artifact)
    }

    pub fn classify_artifact_reload(&self, artifact: &AssetArtifactDescriptor) -> ReloadStatus {
        let subject = self
            .catalog
            .asset(artifact.asset_id)
            .map(|record| ReloadSubject::asset(record.asset_id, record.kind, record.revision_id))
            .unwrap_or_else(|| {
                ReloadSubject::new(editor_preview::ReloadSubjectKind::Asset, "orphan artifact")
            });
        let product = runtime_product_ref_for_artifact(artifact);
        match artifact.validity {
            ArtifactValidity::Valid | ArtifactValidity::Stale => ReloadStatus::new(
                subject,
                reload_decision_for_kind(artifact.kind),
                reload_message_for_kind(artifact.kind),
            )
            .with_runtime_product(product),
            ArtifactValidity::FailedPreserved => ReloadStatus::new(
                subject,
                ReloadDecision::FailedPreserved,
                "reload failed; preserving the prior valid runtime product",
            )
            .with_prior_valid_product(product),
            ArtifactValidity::Rejected => ReloadStatus::new(
                subject,
                ReloadDecision::Rejected,
                "reload artifact was rejected by ratification",
            ),
        }
    }

    pub fn record_reload_status(&mut self, status: ReloadStatus) {
        self.reload_statuses.push(status);
    }

    pub fn reload_statuses(&self) -> &[ReloadStatus] {
        &self.reload_statuses
    }

    pub fn drain_reload_statuses(&mut self) -> Vec<ReloadStatus> {
        self.reload_statuses.drain(..).collect()
    }

    pub fn select_asset(&mut self, asset_id: Option<AssetId>) {
        self.selected_asset_id = asset_id;
    }

    pub fn selected_asset_id(&self) -> Option<AssetId> {
        self.selected_asset_id
    }

    pub fn set_selected_field_product(&mut self, descriptor: Option<FieldProductDescriptor>) {
        self.selected_field_product = descriptor;
    }

    pub fn selected_field_product(&self) -> Option<&FieldProductDescriptor> {
        self.selected_field_product.as_ref()
    }

    pub fn asset_summary_lines(&self) -> Vec<String> {
        let mut lines = self
            .catalog
            .assets()
            .map(|record| {
                let marker = if Some(record.asset_id) == self.selected_asset_id {
                    "*"
                } else {
                    " "
                };
                let state = if self.dirty_assets.contains(&record.asset_id) {
                    "dirty"
                } else {
                    "current"
                };
                format!(
                    "{marker} {} [{:?}] artifacts={} {state}",
                    record.display_name,
                    record.kind,
                    record.artifact_ids.len()
                )
            })
            .collect::<Vec<_>>();
        if lines.is_empty() {
            lines.push("No catalog assets".to_string());
        }
        lines
    }

    pub fn import_diagnostic_lines(&self) -> Vec<String> {
        let mut lines = self
            .diagnostics
            .iter()
            .map(|diagnostic| {
                format!(
                    "{:?} {:?}: {}",
                    diagnostic.severity, diagnostic.code, diagnostic.message
                )
            })
            .collect::<Vec<_>>();
        if lines.is_empty() {
            lines.push("No asset import diagnostics".to_string());
        }
        lines
    }

    pub fn reload_status_lines(&self) -> Vec<String> {
        let mut lines = self
            .reload_statuses
            .iter()
            .map(|status| {
                format!(
                    "reload {:?} {}: {}",
                    status.decision, status.subject.label, status.message
                )
            })
            .collect::<Vec<_>>();
        if lines.is_empty() {
            lines.push("No runtime reload status".to_string());
        }
        lines
    }

    pub fn field_product_lines(&self) -> Vec<String> {
        if let Some(product) = &self.selected_field_product {
            return vec![
                format!("product: {}", product.product_id.0),
                format!("kind: {:?}", product.kind),
                format!("freshness: {:?}", product.freshness),
                format!("scale: {}", product.scale_band),
                format!("chunks: {}", product.scope.chunk_ids.len()),
                format!("regions: {}", product.scope.region_ids.len()),
                format!("producer: {}", product.lineage.producer),
            ];
        }
        let formed = self
            .catalog
            .artifacts
            .values()
            .filter(|artifact| artifact.kind == AssetKind::FormedFieldProduct)
            .count();
        vec![
            "No selected field product".to_string(),
            format!("formed field artifacts: {formed}"),
        ]
    }

    pub fn material_graph_asset_lines(&self) -> Vec<String> {
        let mut lines = self
            .catalog
            .assets()
            .filter(|record| record.kind == AssetKind::MaterialGraph)
            .map(|record| {
                let state = if self.dirty_assets.contains(&record.asset_id) {
                    "dirty"
                } else {
                    "current"
                };
                format!(
                    "material graph asset: {} [{}] artifacts={} {state}",
                    record.display_name,
                    record.stable_name,
                    record.artifact_ids.len()
                )
            })
            .collect::<Vec<_>>();
        if lines.is_empty() {
            lines.push("No source-backed material graph assets".to_string());
        }
        lines
    }

    pub fn material_product_lines(&self) -> Vec<String> {
        let mut lines = self
            .catalog
            .artifacts
            .values()
            .filter_map(|artifact| match &artifact.payload_kind {
                ArtifactPayloadKind::FormedMaterialProduct { product_id } => Some(format!(
                    "formed material product: {product_id} [{:?}] validity={:?} cache={}",
                    artifact.kind,
                    artifact.validity,
                    artifact.cache_key.as_str()
                )),
                _ => None,
            })
            .collect::<Vec<_>>();
        if lines.is_empty() {
            lines.push("No formed material products".to_string());
        }
        lines
    }

    pub fn texture_product_lines(&self) -> Vec<String> {
        let mut lines = self
            .catalog
            .artifacts
            .values()
            .filter(|artifact| !artifact_is_volume_texture(artifact))
            .filter_map(texture_artifact_line)
            .collect::<Vec<_>>();
        if lines.is_empty() {
            lines.push("No Texture2D or generated texture products".to_string());
        }
        lines
    }

    pub fn volume_texture_product_lines(&self) -> Vec<String> {
        let mut lines = self
            .catalog
            .artifacts
            .values()
            .filter(|artifact| artifact_is_volume_texture(artifact))
            .filter_map(texture_artifact_line)
            .collect::<Vec<_>>();
        if lines.is_empty() {
            lines.push("No Texture3D or volume texture products".to_string());
        }
        lines
    }

    pub fn texture_preview_descriptor(&self) -> Option<TexturePreviewDescriptor> {
        first_texture_product_id(self, false).map(TexturePreviewDescriptor::new)
    }

    pub fn volume_texture_preview_descriptor(&self) -> Option<TexturePreviewDescriptor> {
        first_texture_product_id(self, true).map(TexturePreviewDescriptor::new)
    }

    pub fn sdf_brush_lines(&self) -> Vec<String> {
        let mut lines = self
            .catalog
            .assets()
            .filter(|record| record.kind == AssetKind::SdfBrushLayer)
            .map(|record| format!("{} [{}]", record.display_name, record.stable_name))
            .collect::<Vec<_>>();
        if lines.is_empty() {
            lines.push("No SDF brush layer assets".to_string());
        }
        lines
    }

    pub fn has_stale_field_product(&self) -> bool {
        self.selected_field_product
            .as_ref()
            .map(|product| product.freshness == FieldProductFreshness::PotentiallyStale)
            .unwrap_or(false)
    }
}

fn texture_artifact_line(artifact: &AssetArtifactDescriptor) -> Option<String> {
    match &artifact.payload_kind {
        ArtifactPayloadKind::TextureProduct {
            product_id,
            dimension,
        } => Some(format!(
            "texture product: {product_id} dimension={dimension} [{:?}] validity={:?} cache={}",
            artifact.kind,
            artifact.validity,
            artifact.cache_key.as_str()
        )),
        ArtifactPayloadKind::GeneratedTextureProduct { product_id } => Some(format!(
            "generated texture product: {product_id} [{:?}] validity={:?} cache={}",
            artifact.kind,
            artifact.validity,
            artifact.cache_key.as_str()
        )),
        _ => None,
    }
}

fn artifact_is_volume_texture(artifact: &AssetArtifactDescriptor) -> bool {
    artifact.kind == AssetKind::Texture3DVolume
        || matches!(
            &artifact.payload_kind,
            ArtifactPayloadKind::TextureProduct { dimension, .. }
                if dimension.contains("3D") || dimension.contains("Volume")
        )
}

fn first_texture_product_id(
    runtime: &AssetCatalogRuntime,
    volume_only: bool,
) -> Option<TextureProductId> {
    runtime
        .catalog
        .artifacts
        .values()
        .filter(|artifact| artifact_is_volume_texture(artifact) == volume_only)
        .filter_map(|artifact| match &artifact.payload_kind {
            ArtifactPayloadKind::TextureProduct { product_id, .. }
            | ArtifactPayloadKind::GeneratedTextureProduct { product_id } => {
                product_id.parse::<u64>().ok().map(TextureProductId::new)
            }
            _ => None,
        })
        .next()
}

fn reload_decision_for_kind(kind: AssetKind) -> ReloadDecision {
    match kind {
        AssetKind::Scene
        | AssetKind::SdfGraph
        | AssetKind::SdfBrushLayer
        | AssetKind::FieldWorldDefinition
        | AssetKind::WorldEditLog
        | AssetKind::FieldMaterialChannelSet
        | AssetKind::FormedFieldProduct
        | AssetKind::WorldSdfChunkPageArtifact
        | AssetKind::ClipmapBrickmapProduct
        | AssetKind::Shader
        | AssetKind::UiLayout
        | AssetKind::UiDefinition
        | AssetKind::Theme
        | AssetKind::Menu
        | AssetKind::Shortcut
        | AssetKind::WorkspaceDefinition
        | AssetKind::EditorDefinition => ReloadDecision::LiveReload,
        AssetKind::Prefab
        | AssetKind::Material
        | AssetKind::ProceduralMaterial
        | AssetKind::ProceduralTexture
        | AssetKind::Texture2D
        | AssetKind::Texture3DVolume
        | AssetKind::ForeignMeshReferenceArtifact => ReloadDecision::PreviewSessionRestartRequired,
        AssetKind::ForeignMeshReferenceSource => ReloadDecision::RuntimeProcessRestartRequired,
        AssetKind::MaterialGraph
        | AssetKind::GameplayGraph
        | AssetKind::GameplayRuleTrigger
        | AssetKind::GameplayAbility
        | AssetKind::GameplayQuest
        | AssetKind::GameplayAtrIrProduct
        | AssetKind::GameplayEcsLoweringProduct
        | AssetKind::ParticleGraph
        | AssetKind::ParticleEmitter
        | AssetKind::PhysicsConfig
        | AssetKind::AnimationClip
        | AssetKind::AnimationGraph
        | AssetKind::ProcgenGraph
        | AssetKind::Graph
        | AssetKind::Script => ReloadDecision::Unsupported,
        AssetKind::DiagnosticsCapture => ReloadDecision::Rejected,
    }
}

fn reload_message_for_kind(kind: AssetKind) -> &'static str {
    match reload_decision_for_kind(kind) {
        ReloadDecision::LiveReload => "asset revision is live-reloadable",
        ReloadDecision::PreviewSessionRestartRequired => {
            "asset revision requires a preview-session restart"
        }
        ReloadDecision::RuntimeProcessRestartRequired => {
            "asset revision requires a runtime-process restart"
        }
        ReloadDecision::Unsupported => "asset kind does not have an M5 runtime reload contract",
        ReloadDecision::FailedPreserved => "reload failed; preserving prior valid product",
        ReloadDecision::Rejected => "asset revision was rejected",
    }
}

fn runtime_product_ref_for_artifact(artifact: &AssetArtifactDescriptor) -> RuntimeProductRef {
    RuntimeProductRef::new(
        runtime_product_kind_for_asset(artifact.kind),
        artifact.kind_string(),
    )
    .for_asset(artifact.asset_id, artifact.kind)
    .with_artifact(artifact.artifact_id, artifact.artifact_revision_id)
}

fn runtime_product_kind_for_asset(kind: AssetKind) -> RuntimeProductKind {
    match kind {
        AssetKind::Scene => RuntimeProductKind::Scene,
        AssetKind::FormedFieldProduct => RuntimeProductKind::FieldProduct,
        AssetKind::WorldSdfChunkPageArtifact | AssetKind::ClipmapBrickmapProduct => {
            RuntimeProductKind::WorldSdfPayload
        }
        AssetKind::Material | AssetKind::ProceduralMaterial => RuntimeProductKind::Material,
        AssetKind::ProceduralTexture | AssetKind::Texture2D | AssetKind::Texture3DVolume => {
            RuntimeProductKind::Texture
        }
        AssetKind::Shader => RuntimeProductKind::Shader,
        AssetKind::UiDefinition | AssetKind::UiLayout => RuntimeProductKind::UiDefinition,
        _ => RuntimeProductKind::UnsupportedFutureKind,
    }
}

trait AssetKindLabel {
    fn kind_string(&self) -> String;
}

impl AssetKindLabel for AssetArtifactDescriptor {
    fn kind_string(&self) -> String {
        format!("{:?}", self.kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{
        ArtifactCacheKey, ArtifactPayloadKind, AssetArtifactDescriptor, AssetRecord,
        asset_artifact_id, asset_id,
    };

    #[test]
    fn reload_classification_marks_current_m5_kinds_live() {
        let mut runtime = AssetCatalogRuntime::new();
        let asset_id = asset_id(1);
        let artifact_id = asset_artifact_id(2);
        runtime.catalog_mut().insert_asset_record(AssetRecord::new(
            asset_id,
            "scene",
            "Scene",
            AssetKind::Scene,
        ));
        runtime
            .catalog_mut()
            .insert_artifact(AssetArtifactDescriptor::new(
                artifact_id,
                asset_id,
                AssetKind::Scene,
                ArtifactPayloadKind::SceneManifest,
                ArtifactCacheKey::new("scene"),
            ));
        runtime.mark_asset_dirty(asset_id);

        let statuses = runtime.classify_dirty_assets_for_reload();

        assert_eq!(statuses[0].decision, ReloadDecision::LiveReload);
        assert_eq!(runtime.reload_statuses().len(), 1);
        assert!(
            runtime.reload_status_lines()[0].contains("LiveReload"),
            "existing surfaces should expose reload decision text"
        );
    }

    #[test]
    fn reload_classification_keeps_future_domains_unsupported() {
        assert_eq!(
            reload_decision_for_kind(AssetKind::MaterialGraph),
            ReloadDecision::Unsupported
        );
        assert_eq!(
            reload_decision_for_kind(AssetKind::GameplayGraph),
            ReloadDecision::Unsupported
        );
        assert_eq!(
            reload_decision_for_kind(AssetKind::Script),
            ReloadDecision::Unsupported
        );
    }

    #[test]
    fn failed_reload_preserves_prior_valid_artifact() {
        let artifact = AssetArtifactDescriptor::new(
            asset_artifact_id(5),
            asset_id(1),
            AssetKind::Shader,
            ArtifactPayloadKind::ShaderMetadata,
            ArtifactCacheKey::new("shader"),
        )
        .with_validity(ArtifactValidity::FailedPreserved);
        let runtime = AssetCatalogRuntime::new();

        let status = runtime.classify_artifact_reload(&artifact);

        assert_eq!(status.decision, ReloadDecision::FailedPreserved);
        assert!(status.prior_valid_product.is_some());
    }
}
