//! File: domain/ui/ui_program/src/graphs/mod.rs
//! Crate: ui_program

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlGraph {
    pub nodes: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutGraph {
    pub constraints: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateGraph {
    pub requirements: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleGraph {
    pub rules: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionGraph {
    pub handlers: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingGraph {
    pub bindings: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualGraph {
    pub operators: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityGraph {
    pub nodes: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionGraph {
    pub entries: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiProgramGraphs {
    pub control: ControlGraph,
    pub layout: LayoutGraph,
    pub state: StateGraph,
    pub style: StyleGraph,
    pub interaction: InteractionGraph,
    pub binding: BindingGraph,
    pub visual: VisualGraph,
    pub accessibility: AccessibilityGraph,
    pub inspection: InspectionGraph,
}
