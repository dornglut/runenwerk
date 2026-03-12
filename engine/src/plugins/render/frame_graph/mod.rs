use super::pipelines::PipelineKey;
use anyhow::{Result, anyhow, bail};
use std::collections::{BTreeMap, BTreeSet};

mod builders;
mod executor;
mod executor_contexts;
mod executor_registry;
mod ids;
mod registry;
mod resources;
mod spec;
mod validation;

pub use builders::*;
pub use executor::*;
pub use executor_contexts::*;
pub use executor_registry::*;
pub use ids::*;
pub use registry::*;
pub use resources::*;
pub use spec::*;
pub use validation::*;

#[cfg(test)]
mod tests {
    use super::{
        OwnerRenderGraphRegistration, RegisteredPassDescriptor, RegisteredPipelineDescriptor,
        RenderFeatureGraphSpec, RenderGraphRegistryResource,
    };

    #[test]
    fn upsert_owner_replaces_existing_owner_registration() {
        let mut registry = RenderGraphRegistryResource::default();
        registry.upsert_owner(
            OwnerRenderGraphRegistration::new("sdf").with_pipelines(vec![
                RegisteredPipelineDescriptor::new("sdf.compute", "world_compute_sdf_3d".into()),
            ]),
        );
        registry.upsert_owner(
            OwnerRenderGraphRegistration::new("sdf")
                .with_passes(vec![RegisteredPassDescriptor::compute("sdf_compute")]),
        );
        assert_eq!(registry.owner_count(), 1);
        let owners = registry.owners();
        let owner = &owners[0];
        assert!(owner.pipelines.is_empty());
        assert_eq!(owner.passes.len(), 1);
    }

    #[test]
    fn builder_builds_feature_graph_and_converts_to_owner_registration() {
        let spec = RenderFeatureGraphSpec::builder("sdf_renderer")
            .resource("sdf.params")
            .resource("sdf.color")
            .resource("surface.color")
            .pipeline_compute(
                "sdf.compute.raymarch",
                "assets/shaders/sdf_compute_3d_example.wgsl",
            )
            .pipeline_render_builtin("sdf.compose.fullscreen", "compose.fullscreen")
            .compute_pass("sdf.compute")
            .pipeline("sdf.compute.raymarch")
            .executor_builtin_compute()
            .reads(["sdf.params"])
            .writes(["sdf.color"])
            .finish()
            .render_pass("sdf.compose")
            .pipeline("sdf.compose.fullscreen")
            .executor_builtin_compose()
            .reads(["sdf.color"])
            .writes(["surface.color"])
            .depends_on(["sdf.compute"])
            .finish()
            .build()
            .expect("feature graph should build");

        let owner = spec.clone().into_owner_registration();
        assert_eq!(owner.owner, "sdf_renderer");
        assert_eq!(owner.pipelines.len(), 2);
        assert_eq!(owner.passes.len(), 2);
        assert_eq!(owner.passes[0].id, "sdf.compute");
        assert_eq!(owner.passes[0].executor.as_deref(), Some("builtin_compute"));
    }

    #[test]
    fn register_feature_graph_replaces_existing_registration() {
        let spec_a = RenderFeatureGraphSpec::builder("sdf_renderer")
            .resource("sdf.params")
            .pipeline_compute(
                "sdf.compute.a",
                "assets/shaders/sdf_compute_3d_example.wgsl",
            )
            .compute_pass("sdf.compute")
            .pipeline("sdf.compute.a")
            .executor_builtin_compute()
            .reads(["sdf.params"])
            .finish()
            .build()
            .expect("spec_a should build");
        let spec_b = RenderFeatureGraphSpec::builder("sdf_renderer")
            .resource("sdf.color")
            .pipeline_render_builtin("sdf.compose.fullscreen", "compose.fullscreen")
            .render_pass("sdf.compose")
            .pipeline("sdf.compose.fullscreen")
            .executor_builtin_compose()
            .reads(["sdf.color"])
            .finish()
            .build()
            .expect("spec_b should build");

        let mut registry = RenderGraphRegistryResource::default();
        registry.register_feature_graph(spec_a);
        registry.register_feature_graph(spec_b);

        assert_eq!(registry.owner_count(), 1);
        let owners = registry.owners();
        assert_eq!(owners[0].owner, "sdf_renderer");
        assert_eq!(owners[0].passes.len(), 1);
        assert_eq!(owners[0].passes[0].id, "sdf.compose");
    }
}
