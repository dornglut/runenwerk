use crate::plugins::render::api::{is_namespaced_id, namespace_of};
use crate::plugins::render::composition::RenderFlowContribution;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RenderFlowVariant {
    #[default]
    MainView,
    EditorViewport(String),
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FragmentResourceSpec {
    SampledTexture {
        id: String,
    },
    StorageTexture {
        id: String,
        #[serde(default)]
        transient: bool,
    },
    ColorTarget {
        id: String,
        #[serde(default)]
        transient: bool,
    },
    DepthTarget {
        id: String,
        #[serde(default)]
        transient: bool,
    },
    HistoryTexture {
        id: String,
    },
    ImportedTexture {
        id: String,
    },
    ImportedBuffer {
        id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentPassKind {
    Compute,
    Fullscreen,
    BuiltinUiComposite,
    Graphics,
    Copy,
    Present,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentPassSpec {
    pub id: String,
    pub kind: FragmentPassKind,
    #[serde(default)]
    pub shader: Option<String>,
    #[serde(default)]
    pub reads: Vec<String>,
    #[serde(default)]
    pub writes: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub sampled_textures: Vec<String>,
    #[serde(default)]
    pub write_textures: Vec<String>,
    #[serde(default)]
    pub vertex_buffers: Vec<String>,
    #[serde(default)]
    pub index_buffers: Vec<String>,
    #[serde(default)]
    pub instance_buffers: Vec<String>,
    #[serde(default)]
    pub indirect_buffers: Vec<String>,
    #[serde(default)]
    pub depth_target: Option<String>,
    #[serde(default)]
    pub workgroup_size: Option<[u32; 3]>,
    #[serde(default)]
    pub clear_color: Option<[f32; 4]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderFlowFragmentSpec {
    pub namespace: String,
    #[serde(default)]
    pub variant: RenderFlowVariant,
    #[serde(default)]
    pub resources: Vec<FragmentResourceSpec>,
    #[serde(default)]
    pub passes: Vec<FragmentPassSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentSpecError {
    pub issues: Vec<String>,
}

impl std::fmt::Display for FragmentSpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.issues.join("; "))
    }
}

impl std::error::Error for FragmentSpecError {}

pub fn parse_fragment_ron(source: &str) -> Result<RenderFlowFragmentSpec, FragmentSpecError> {
    ron::from_str(source).map_err(|err| FragmentSpecError {
        issues: vec![format!("failed to parse render flow fragment: {err}")],
    })
}

impl RenderFlowFragmentSpec {
    pub fn contribution_id(&self) -> String {
        match &self.variant {
            RenderFlowVariant::MainView => self.namespace.clone(),
            RenderFlowVariant::EditorViewport(viewport) => {
                format!("{}_editor_{}", self.namespace, sanitize_variant(viewport))
            }
            RenderFlowVariant::Named(name) => {
                format!("{}_{}", self.namespace, sanitize_variant(name))
            }
        }
    }

    pub fn validate(&self) -> Result<(), FragmentSpecError> {
        let mut issues = Vec::<String>::new();
        if !is_valid_namespace(self.namespace.as_str()) {
            issues.push(format!(
                "invalid fragment namespace '{}': only ASCII alphanumeric, '_' and '-' are allowed",
                self.namespace
            ));
        }

        for resource in &self.resources {
            let id = resource.id();
            validate_owned_id(
                id,
                "resource",
                self.namespace.as_str(),
                resource.is_imported(),
                &mut issues,
            );
        }
        for pass in &self.passes {
            validate_owned_id(
                pass.id.as_str(),
                "pass",
                self.namespace.as_str(),
                false,
                &mut issues,
            );
            for dep in &pass.depends_on {
                if !is_namespaced_id(dep) {
                    issues.push(format!(
                        "dependency id '{}' in pass '{}' must use namespaced format",
                        dep, pass.id
                    ));
                }
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(FragmentSpecError { issues })
        }
    }

    pub fn to_contribution(&self) -> Result<RenderFlowContribution, FragmentSpecError> {
        self.validate()?;

        let mut contribution = RenderFlowContribution::new(self.contribution_id());
        for resource in &self.resources {
            contribution = match resource {
                FragmentResourceSpec::SampledTexture { id } => {
                    contribution.sampled_texture(leak_str(id.clone()))
                }
                FragmentResourceSpec::StorageTexture { id, transient } => {
                    if *transient {
                        contribution.transient_storage_texture(leak_str(id.clone()))
                    } else {
                        contribution.storage_texture(leak_str(id.clone()))
                    }
                }
                FragmentResourceSpec::ColorTarget { id, transient } => {
                    if *transient {
                        contribution.transient_color_target(leak_str(id.clone()))
                    } else {
                        contribution.color_target(leak_str(id.clone()))
                    }
                }
                FragmentResourceSpec::DepthTarget { id, transient } => {
                    if *transient {
                        contribution.transient_depth_target(leak_str(id.clone()))
                    } else {
                        contribution.depth_target(leak_str(id.clone()))
                    }
                }
                FragmentResourceSpec::HistoryTexture { id } => {
                    contribution.history_texture(leak_str(id.clone()))
                }
                FragmentResourceSpec::ImportedTexture { id } => {
                    contribution.import_texture(leak_str(id.clone()))
                }
                FragmentResourceSpec::ImportedBuffer { id } => {
                    contribution.import_buffer(leak_str(id.clone()))
                }
            };
        }

        for pass in &self.passes {
            contribution = apply_pass_spec(contribution, pass);
        }
        Ok(contribution)
    }
}

fn apply_pass_spec(
    mut contribution: RenderFlowContribution,
    spec: &FragmentPassSpec,
) -> RenderFlowContribution {
    let pass_id = leak_str(spec.id.clone());
    match spec.kind {
        FragmentPassKind::Compute => {
            let mut pass = contribution.compute_pass(pass_id);
            if let Some(shader) = &spec.shader {
                pass = pass.shader(leak_str(shader.clone()));
            }
            for id in &spec.reads {
                pass = pass.reads(leak_str(id.clone()));
            }
            for id in &spec.writes {
                pass = pass.writes(leak_str(id.clone()));
            }
            for id in &spec.write_textures {
                pass = pass.write_texture(leak_str(id.clone()));
            }
            for id in &spec.depends_on {
                pass = pass.depends_on(leak_str(id.clone()));
            }
            if let Some([x, y, z]) = spec.workgroup_size {
                pass = pass.workgroup_size(x, y, z);
            }
            contribution = pass.finish();
        }
        FragmentPassKind::Fullscreen => {
            let mut pass = contribution.fullscreen_pass(pass_id);
            if let Some(shader) = &spec.shader {
                pass = pass.shader(leak_str(shader.clone()));
            }
            for id in &spec.sampled_textures {
                pass = pass.sample_texture(leak_str(id.clone()));
            }
            for id in &spec.reads {
                pass = pass.reads(leak_str(id.clone()));
            }
            for id in &spec.writes {
                pass = pass.writes(leak_str(id.clone()));
            }
            for id in &spec.write_textures {
                pass = pass.write_texture(leak_str(id.clone()));
            }
            for id in &spec.depends_on {
                pass = pass.depends_on(leak_str(id.clone()));
            }
            if let Some(clear_color) = spec.clear_color {
                pass = pass.clear_color(clear_color);
            }
            contribution = pass.finish();
        }
        FragmentPassKind::BuiltinUiComposite => {
            let mut pass = contribution.builtin_ui_composite_pass(pass_id);
            for id in &spec.reads {
                pass = pass.reads(leak_str(id.clone()));
            }
            for id in &spec.writes {
                pass = pass.writes(leak_str(id.clone()));
            }
            for id in &spec.depends_on {
                pass = pass.depends_on(leak_str(id.clone()));
            }
            contribution = pass.finish();
        }
        FragmentPassKind::Graphics => {
            let mut pass = contribution.graphics_pass(pass_id);
            if let Some(shader) = &spec.shader {
                pass = pass.shader(leak_str(shader.clone()));
            }
            for id in &spec.sampled_textures {
                pass = pass.sample_texture(leak_str(id.clone()));
            }
            for id in &spec.reads {
                pass = pass.reads(leak_str(id.clone()));
            }
            for id in &spec.writes {
                pass = pass.writes(leak_str(id.clone()));
            }
            for id in &spec.write_textures {
                pass = pass.write_texture(leak_str(id.clone()));
            }
            for id in &spec.vertex_buffers {
                pass = pass.vertex_buffer(leak_str(id.clone()));
            }
            for id in &spec.index_buffers {
                pass = pass.index_buffer(leak_str(id.clone()));
            }
            for id in &spec.instance_buffers {
                pass = pass.instance_buffer(leak_str(id.clone()));
            }
            for id in &spec.indirect_buffers {
                pass = pass.indirect_buffer(leak_str(id.clone()));
            }
            if let Some(depth_target) = &spec.depth_target {
                pass = pass.depth_target(leak_str(depth_target.clone()));
            }
            for id in &spec.depends_on {
                pass = pass.depends_on(leak_str(id.clone()));
            }
            contribution = pass.finish();
        }
        FragmentPassKind::Copy => {
            let mut pass = contribution.copy_pass(pass_id);
            for id in &spec.reads {
                pass = pass.reads(leak_str(id.clone()));
            }
            for id in &spec.writes {
                pass = pass.writes(leak_str(id.clone()));
            }
            for id in &spec.depends_on {
                pass = pass.depends_on(leak_str(id.clone()));
            }
            contribution = pass.finish();
        }
        FragmentPassKind::Present => {
            let mut pass = contribution.present_pass(pass_id);
            for id in &spec.reads {
                pass = pass.reads(leak_str(id.clone()));
            }
            for id in &spec.depends_on {
                pass = pass.depends_on(leak_str(id.clone()));
            }
            contribution = pass.finish();
        }
    }

    contribution
}

fn validate_owned_id(
    id: &str,
    label: &str,
    namespace: &str,
    allow_external_namespace: bool,
    issues: &mut Vec<String>,
) {
    if !is_namespaced_id(id) {
        issues.push(format!(
            "{} id '{}' must use namespaced format (for example '{}.item')",
            label, id, namespace
        ));
        return;
    }

    let Some(owner_namespace) = namespace_of(id) else {
        issues.push(format!("{} id '{}' must include namespace", label, id));
        return;
    };
    if !allow_external_namespace && owner_namespace != namespace {
        issues.push(format!(
            "{} id '{}' must stay in fragment namespace '{}'",
            label, id, namespace
        ));
    }
}

fn is_valid_namespace(namespace: &str) -> bool {
    !namespace.is_empty()
        && namespace
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn sanitize_variant(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn leak_str(value: String) -> &'static str {
    // RenderFlow currently accepts &'static str IDs; asset-authored fragments
    // are bridged via interned leaked strings until owned-ID API variants exist.
    Box::leak(value.into_boxed_str())
}

impl FragmentResourceSpec {
    fn id(&self) -> &str {
        match self {
            FragmentResourceSpec::SampledTexture { id }
            | FragmentResourceSpec::StorageTexture { id, .. }
            | FragmentResourceSpec::ColorTarget { id, .. }
            | FragmentResourceSpec::DepthTarget { id, .. }
            | FragmentResourceSpec::HistoryTexture { id }
            | FragmentResourceSpec::ImportedTexture { id }
            | FragmentResourceSpec::ImportedBuffer { id } => id.as_str(),
        }
    }

    fn is_imported(&self) -> bool {
        matches!(
            self,
            FragmentResourceSpec::ImportedTexture { .. }
                | FragmentResourceSpec::ImportedBuffer { .. }
        )
    }
}
