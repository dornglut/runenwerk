//! Crate: material_graph
//! Purpose: Domain contracts for authored material graphs, ratification, lowering, and formed material products.

pub mod authored;
pub mod catalog;
pub mod commands;
pub mod formed;
pub mod ids;
pub mod ir;
pub mod lowering;
pub mod persistence;
pub mod ratification;

pub use authored::{
    MaterialGraphDocument, MaterialGraphEditorMetadata, MaterialGraphEditorState,
    MaterialGraphLayoutGroup, MaterialGraphNodeLayout, MaterialGraphPreviewFixture,
    MaterialGraphPreviewSelection, MaterialGraphViewportState, MaterialOutputTarget,
};
pub use catalog::{
    MaterialInputContract, MaterialLiteral, MaterialNodeCatalog, MaterialNodeCatalogError,
    MaterialNodeDescriptor, MaterialOutputContract, MaterialResourceContract, MaterialResourceKind,
    MaterialValueContract, MaterialValueType,
};
pub use commands::{
    MaterialGraphCommandError, add_catalog_node, connect_ports, delete_selection, disconnect_edge,
    move_node_layout, set_node_text_value, set_node_texture_resource,
};
pub use formed::{
    FormedMaterialProduct, MaterialCacheKey, MaterialParameterDescriptor, MaterialParameterKind,
    MaterialSourceMap, MaterialSourceMapEntry, MaterialSpecializationFragment,
};
pub use ids::{MaterialGraphDocumentId, MaterialProductId};
pub use ir::{
    MATERIAL_GRAPH_VALUE_TEXTURE_REF, MATERIAL_IR_CONTRACT_VERSION, MaterialIr, MaterialIrEdge,
    MaterialIrInput, MaterialIrInputSource, MaterialIrNode, MaterialIrOutput, MaterialIrValue,
    MaterialNodeOp, MaterialResourceBinding,
};
pub use lowering::{MaterialLoweringResult, lower_material_graph};
pub use persistence::{
    MATERIAL_GRAPH_SOURCE_FILE_VERSION_V1, MATERIAL_GRAPH_SOURCE_FILE_VERSION_V2,
    MaterialGraphSourceFileV1, MaterialGraphSourceFileV2, MaterialGraphSourceIssue,
};
pub use ratification::{
    MaterialGraphIssueCode, MaterialGraphIssueSubject, MaterialGraphRatificationReport,
    ratify_material_graph,
};
