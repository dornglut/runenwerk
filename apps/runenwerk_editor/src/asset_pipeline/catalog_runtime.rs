use std::collections::BTreeSet;

use asset::{AssetCatalog, AssetDiagnosticRecord, AssetId, AssetKind};
use world_sdf::{FieldProductDescriptor, FieldProductFreshness};

#[derive(Debug, Clone, Default)]
pub struct AssetCatalogRuntime {
    catalog: AssetCatalog,
    diagnostics: Vec<AssetDiagnosticRecord>,
    dirty_assets: BTreeSet<AssetId>,
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
