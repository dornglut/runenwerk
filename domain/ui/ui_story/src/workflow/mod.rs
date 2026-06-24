//! Semantic workflow graph contracts for UI Story V2.
//!
//! This module owns the durable proof graph shape only. It does not load files,
//! run compilers, execute renderers, mount previews, or own app/editor behavior.

mod builtin;
mod edge;
mod graph;
mod node;
mod profile;
mod topo;
mod validate;

pub use builtin::{UiStoryBuiltinWorkflowProfile, UiStoryBuiltinWorkflowProfiles};
pub use edge::{UiStoryWorkflowDependency, UiStoryWorkflowEdge};
pub use graph::UiStoryWorkflowGraph;
pub use node::{UiStoryWorkflowNode, UiStoryWorkflowNodePolicy};
pub use profile::UiStoryWorkflowProfile;
pub use topo::UiStoryWorkflowTopologyError;
pub use validate::validate_workflow_graph;
