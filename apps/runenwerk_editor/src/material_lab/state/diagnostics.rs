use super::*;

impl MaterialLabRuntime {
    pub(super) fn material_diagnostic_rows(&self) -> Vec<MaterialDiagnosticRowViewModel> {
        self.diagnostics
            .iter()
            .map(|diagnostic| MaterialDiagnosticRowViewModel {
                severity: material_diagnostic_severity(diagnostic.severity),
                code: diagnostic.code.diagnostic_code().as_str().to_string(),
                subject_label: diagnostic.subject.clone(),
                category_label: Some("material workflow".to_string()),
                message: diagnostic.message.clone(),
            })
            .collect()
    }

    pub(super) fn preview_scene_product_diagnostic_rows(
        &self,
    ) -> Vec<MaterialDiagnosticRowViewModel> {
        let Some(reason) = self.preview_scene_product_failure_reason() else {
            return Vec::new();
        };
        let code = preview_scene_product_failure_code(&reason)
            .unwrap_or("material.preview_scene.failure")
            .to_string();
        vec![MaterialDiagnosticRowViewModel {
            severity: MaterialDiagnosticSeverity::Error,
            code,
            subject_label: None,
            category_label: Some("preview scene product".to_string()),
            message: reason,
        }]
    }

    pub(super) fn preview_scene_product_failure_reason(&self) -> Option<String> {
        if matches!(
            self.preview_scene_product_status(),
            PreviewSceneProductRuntimeStatus::Current { .. }
        ) {
            return None;
        }
        self.diagnostics
            .iter()
            .rev()
            .find(|diagnostic| preview_scene_product_failure_code(&diagnostic.message).is_some())
            .map(|diagnostic| diagnostic.message.clone())
    }

    pub(super) fn material_resource_binding_diagnostic_rows(
        &self,
        catalog: &AssetCatalog,
    ) -> Vec<MaterialResourceBindingDiagnosticViewModel> {
        let mut rows = Vec::new();
        if let Some((_, document)) = self.active_source_document() {
            let node_catalog = material_graph::MaterialNodeCatalog::first_slice();
            for node in &document.graph.nodes {
                let Some(descriptor) = node_catalog.descriptor(&node.name) else {
                    continue;
                };
                for resource in &descriptor.resources {
                    match node.value(&resource.key) {
                        Some(graph::GraphValue::Resource(reference)) => {
                            let binding = MaterialResourceBinding::new(
                                node.id,
                                resource.key.clone(),
                                reference.clone(),
                            );
                            rows.push(material_resource_binding_diagnostic_row(catalog, &binding));
                        }
                        Some(_) => rows.push(material_resource_binding_unresolved_row(
                            MaterialResourceBindingStatusKind::Incompatible,
                            "material.resource.non_resource_value",
                            node.id,
                            &resource.key,
                            Some(resource.kind.label()),
                            "resource slot contains a non-resource graph value",
                        )),
                        None => rows.push(material_resource_binding_unresolved_row(
                            MaterialResourceBindingStatusKind::Unresolved,
                            "material.resource.unresolved_binding",
                            node.id,
                            &resource.key,
                            Some(resource.kind.label()),
                            "resource slot has no texture reference",
                        )),
                    }
                }
            }
        }

        if rows.is_empty() {
            if let Some(preview) = &self.active_preview {
                rows.extend(preview.resolved_resources.iter().map(|resource| {
                    let status = catalog
                        .artifact(resource.artifact_id)
                        .map(|artifact| match artifact.payload_kind {
                            ArtifactPayloadKind::GeneratedTextureProduct { .. } => {
                                MaterialResourceBindingStatusKind::GeneratedAvailable
                            }
                            _ => MaterialResourceBindingStatusKind::Resolved,
                        })
                        .unwrap_or(MaterialResourceBindingStatusKind::Resolved);
                    material_resource_binding_resolved_row(resource, status)
                }));
            }
        }

        rows
    }

    pub(super) fn diagnostic_lines(&self) -> Vec<String> {
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
        if let Some(status) = &self.last_workflow_status {
            lines.push(format!("last material workflow: {status}"));
        }
        if lines.is_empty() {
            lines.push("No material diagnostics".to_string());
        }
        lines
    }
}

fn material_diagnostic_severity(severity: AssetDiagnosticSeverity) -> MaterialDiagnosticSeverity {
    match severity {
        AssetDiagnosticSeverity::Info => MaterialDiagnosticSeverity::Info,
        AssetDiagnosticSeverity::Warning => MaterialDiagnosticSeverity::Warning,
        AssetDiagnosticSeverity::Error => MaterialDiagnosticSeverity::Error,
        AssetDiagnosticSeverity::Fatal => MaterialDiagnosticSeverity::Fatal,
    }
}

fn preview_scene_product_failure_code(message: &str) -> Option<&'static str> {
    let normalized = message.to_ascii_lowercase();
    if normalized.contains("generated shader bundle")
        && (normalized.contains("requires") || normalized.contains("missing"))
    {
        return Some("material.preview_scene.generated_bundle_missing");
    }
    if normalized.contains("generated shader bundle") && normalized.contains("stale") {
        return Some("material.preview_scene.generated_bundle_stale");
    }
    if normalized.contains("resource layout identity") && normalized.contains("does not match") {
        return Some("material.preview_scene.resource_layout_identity_mismatch");
    }
    if normalized.contains("material table identity") && normalized.contains("does not match") {
        return Some("material.preview_scene.material_table_identity_mismatch");
    }
    if normalized.contains("no resolved source-backed material product")
        || (normalized.contains("scene material slot") && normalized.contains("unresolved"))
    {
        return Some("material.preview_scene.unresolved_scene_slot");
    }
    if normalized.contains("preview scene product") && normalized.contains("stale") {
        return Some("material.preview_scene.stale_product");
    }
    if normalized.contains("table resource slot")
        && normalized.contains("conflicting resource identities")
    {
        return Some("material.preview_scene.resource_slot_identity_conflict");
    }
    if normalized.contains("preview scene product") {
        return Some("material.preview_scene.failure");
    }
    None
}

fn material_resource_binding_unresolved_row(
    status: MaterialResourceBindingStatusKind,
    code: impl Into<String>,
    node_id: graph::NodeId,
    binding_key: &str,
    expected_kind_label: Option<&str>,
    message: impl Into<String>,
) -> MaterialResourceBindingDiagnosticViewModel {
    MaterialResourceBindingDiagnosticViewModel {
        severity: material_resource_binding_severity(status),
        code: code.into(),
        binding_label: format!("node {} resource '{binding_key}'", node_id.raw()),
        resource_key_or_slot_label: binding_key.to_string(),
        expected_kind_label: expected_kind_label.map(str::to_string),
        resolved_artifact_label: None,
        message: message.into(),
        status,
    }
}

fn material_resource_binding_resolved_row(
    resource: &ResolvedMaterialResource,
    status: MaterialResourceBindingStatusKind,
) -> MaterialResourceBindingDiagnosticViewModel {
    MaterialResourceBindingDiagnosticViewModel {
        severity: material_resource_binding_severity(status),
        code: "material.resource.resolved".to_string(),
        binding_label: format!(
            "node {} resource '{}'",
            resource.node_id.raw(),
            resource.binding_key
        ),
        resource_key_or_slot_label: resource.reference.stable_id.as_str().to_string(),
        expected_kind_label: Some(resource.dimension.clone()),
        resolved_artifact_label: Some(format!("artifact {}", resource.artifact_id.raw())),
        message: format!(
            "texture resource resolved to artifact {} ({})",
            resource.artifact_id.raw(),
            resource.residency_identity
        ),
        status,
    }
}

fn material_resource_binding_severity(
    status: MaterialResourceBindingStatusKind,
) -> MaterialDiagnosticSeverity {
    match status {
        MaterialResourceBindingStatusKind::Resolved
        | MaterialResourceBindingStatusKind::GeneratedAvailable => MaterialDiagnosticSeverity::Info,
        MaterialResourceBindingStatusKind::GeneratedUnavailable
        | MaterialResourceBindingStatusKind::Unknown => MaterialDiagnosticSeverity::Warning,
        MaterialResourceBindingStatusKind::Missing
        | MaterialResourceBindingStatusKind::Ambiguous
        | MaterialResourceBindingStatusKind::Incompatible
        | MaterialResourceBindingStatusKind::Unsupported
        | MaterialResourceBindingStatusKind::Unresolved => MaterialDiagnosticSeverity::Error,
    }
}
