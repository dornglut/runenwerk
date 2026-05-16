//! Crate: material_graph
//! Purpose: Domain contracts for authored material graphs, ratification, lowering, and formed material products.

pub mod authored;
pub mod catalog;
pub mod formed;
pub mod ids;
pub mod lowering;
pub mod persistence;
pub mod ratification;

pub use authored::{MaterialGraphDocument, MaterialOutputTarget};
pub use catalog::{MaterialNodeCatalog, MaterialNodeDescriptor};
pub use formed::{
    FormedMaterialProduct, MaterialCacheKey, MaterialParameterDescriptor, MaterialParameterKind,
    MaterialSourceMap, MaterialSourceMapEntry, MaterialSpecializationFragment,
};
pub use ids::{MaterialGraphDocumentId, MaterialProductId};
pub use lowering::{MaterialLoweringResult, lower_material_graph};
pub use persistence::{
    MATERIAL_GRAPH_SOURCE_FILE_VERSION_V1, MaterialGraphSourceFileV1, MaterialGraphSourceIssue,
};
pub use ratification::{
    MaterialGraphIssueCode, MaterialGraphIssueSubject, MaterialGraphRatificationReport,
    ratify_material_graph,
};
