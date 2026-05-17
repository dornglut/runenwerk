//! File: domain/material_graph/src/formed.rs
//! Purpose: Formed material product descriptors and source maps.

use graph::NodeId;
use product::{
    ProductAuthorityClass, ProductConsumerClass, ProductDescriptorCore, ProductFamily,
    ProductFreshness, ProductIdentity, ProductKind, ProductLineage, ProductQueryPolicy,
    ProductRebuildPolicy, ProductResidency, ProductRetentionPolicy, ProductScaleBand, ProductScope,
};

use crate::{MaterialGraphDocumentId, MaterialIr, MaterialOutputTarget, MaterialProductId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialCacheKey(pub String);

impl MaterialCacheKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialSpecializationFragment(pub String);

impl MaterialSpecializationFragment {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialParameterKind {
    Scalar,
    Vector2,
    Vector3,
    Vector4,
    Texture2D,
    Texture3D,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialParameterDescriptor {
    pub key: String,
    pub kind: MaterialParameterKind,
}

impl MaterialParameterDescriptor {
    pub fn new(key: impl Into<String>, kind: MaterialParameterKind) -> Self {
        Self {
            key: key.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialSourceMapEntry {
    pub node_id: NodeId,
    pub role: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct MaterialSourceMap {
    pub entries: Vec<MaterialSourceMapEntry>,
}

impl MaterialSourceMap {
    pub fn from_nodes(nodes: impl IntoIterator<Item = NodeId>) -> Self {
        Self {
            entries: nodes
                .into_iter()
                .map(|node_id| MaterialSourceMapEntry {
                    node_id,
                    role: "semantic_node".to_string(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FormedMaterialProduct {
    pub product_id: MaterialProductId,
    pub product_core: ProductDescriptorCore,
    pub source_document_id: MaterialGraphDocumentId,
    pub output_target: MaterialOutputTarget,
    pub parameters: Vec<MaterialParameterDescriptor>,
    pub source_map: MaterialSourceMap,
    pub specialization_fragment: MaterialSpecializationFragment,
    pub executable_ir: Option<MaterialIr>,
    pub cache_key: MaterialCacheKey,
}

impl FormedMaterialProduct {
    pub fn new(
        product_id: MaterialProductId,
        source_document_id: MaterialGraphDocumentId,
        output_target: MaterialOutputTarget,
        cache_key: MaterialCacheKey,
    ) -> Self {
        let product_core =
            material_product_core(product_id, source_document_id, output_target, &cache_key);
        Self {
            product_id,
            product_core,
            source_document_id,
            output_target,
            parameters: Vec::new(),
            source_map: MaterialSourceMap::default(),
            specialization_fragment: MaterialSpecializationFragment::new("material.first_slice"),
            executable_ir: None,
            cache_key,
        }
    }

    pub fn with_product_id(mut self, product_id: MaterialProductId) -> Self {
        self.product_id = product_id;
        self.product_core = material_product_core(
            self.product_id,
            self.source_document_id,
            self.output_target,
            &self.cache_key,
        );
        self
    }
}

fn material_product_core(
    product_id: MaterialProductId,
    source_document_id: MaterialGraphDocumentId,
    output_target: MaterialOutputTarget,
    cache_key: &MaterialCacheKey,
) -> ProductDescriptorCore {
    let mut descriptor = ProductDescriptorCore::new(
        ProductIdentity::new(product_id.raw()),
        ProductFamily::Material,
        ProductKind::new(material_output_kind(output_target)),
        ProductScope::non_spatial(format!(
            "material_graph_document:{}",
            source_document_id.raw()
        )),
        material_scale_band(output_target),
        ProductLineage::new("material_graph.lowering", 1)
            .with_source_key(format!(
                "material_graph_document:{}",
                source_document_id.raw()
            ))
            .with_source_revision(cache_key.as_str()),
    );
    descriptor.freshness = ProductFreshness::Current;
    descriptor.residency = ProductResidency::NotApplicable;
    descriptor.consumer_class = material_consumer_class(output_target);
    descriptor.authority_class = ProductAuthorityClass::DeterministicDerived;
    descriptor.retention_policy = ProductRetentionPolicy::Cacheable;
    descriptor.rebuild_policy = ProductRebuildPolicy::Budgeted;
    descriptor.query_policy = match output_target {
        MaterialOutputTarget::FieldMaterialChannel => ProductQueryPolicy::CertifiedFallbackAllowed,
        MaterialOutputTarget::PbrPreview | MaterialOutputTarget::RenderMaterial => {
            ProductQueryPolicy::VisualFallbackAllowed
        }
    };
    descriptor
}

fn material_output_kind(output_target: MaterialOutputTarget) -> &'static str {
    match output_target {
        MaterialOutputTarget::PbrPreview => "pbr_preview",
        MaterialOutputTarget::FieldMaterialChannel => "field_material_channel",
        MaterialOutputTarget::RenderMaterial => "render_material",
    }
}

fn material_scale_band(output_target: MaterialOutputTarget) -> ProductScaleBand {
    match output_target {
        MaterialOutputTarget::PbrPreview => ProductScaleBand::Preview,
        MaterialOutputTarget::FieldMaterialChannel => ProductScaleBand::FamilySpecific,
        MaterialOutputTarget::RenderMaterial => ProductScaleBand::Near,
    }
}

fn material_consumer_class(output_target: MaterialOutputTarget) -> ProductConsumerClass {
    match output_target {
        MaterialOutputTarget::PbrPreview => ProductConsumerClass::Editor,
        MaterialOutputTarget::FieldMaterialChannel => ProductConsumerClass::Simulation,
        MaterialOutputTarget::RenderMaterial => ProductConsumerClass::Renderer,
    }
}
