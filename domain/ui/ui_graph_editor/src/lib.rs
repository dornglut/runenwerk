//! File: domain/ui/ui_graph_editor/src/lib.rs
//! Purpose: Backend-neutral graph editor interaction contracts.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GraphNodeKey(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GraphPortKey(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GraphEdgeKey(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPoint {
    pub x: i32,
    pub y: i32,
}

impl GraphPoint {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl GraphRect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn contains(self, point: GraphPoint) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.width
            && point.y < self.y + self.height
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodeBounds {
    pub node: GraphNodeKey,
    pub rect: GraphRect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPortBounds {
    pub port: GraphPortKey,
    pub node: GraphNodeKey,
    pub rect: GraphRect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphHitTarget {
    Background,
    Node(GraphNodeKey),
    Port(GraphPortKey),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GraphHitTestScene {
    pub nodes: Vec<GraphNodeBounds>,
    pub ports: Vec<GraphPortBounds>,
}

impl GraphHitTestScene {
    pub fn hit_test(&self, point: GraphPoint) -> GraphHitTarget {
        for port in self.ports.iter().rev() {
            if port.rect.contains(point) {
                return GraphHitTarget::Port(port.port);
            }
        }
        for node in self.nodes.iter().rev() {
            if node.rect.contains(point) {
                return GraphHitTarget::Node(node.node);
            }
        }
        GraphHitTarget::Background
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphViewport {
    pub pan_x: i32,
    pub pan_y: i32,
    pub zoom_milli: u32,
}

impl Default for GraphViewport {
    fn default() -> Self {
        Self {
            pan_x: 0,
            pan_y: 0,
            zoom_milli: 1000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GraphSelection {
    pub nodes: BTreeSet<GraphNodeKey>,
    pub edges: BTreeSet<GraphEdgeKey>,
}

impl GraphSelection {
    pub fn select_node(node: GraphNodeKey) -> Self {
        Self {
            nodes: BTreeSet::from([node]),
            edges: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEditorViewModel {
    pub viewport: GraphViewport,
    pub selection: GraphSelection,
    pub hit_test_scene: GraphHitTestScene,
    pub can_undo: bool,
    pub can_redo: bool,
    pub active_edit_group: Option<u64>,
}

impl Default for GraphEditorViewModel {
    fn default() -> Self {
        Self {
            viewport: GraphViewport::default(),
            selection: GraphSelection::default(),
            hit_test_scene: GraphHitTestScene::default(),
            can_undo: false,
            can_redo: false,
            active_edit_group: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphEditorAction {
    Pan {
        delta_x: i32,
        delta_y: i32,
    },
    SetZoom {
        zoom_milli: u32,
    },
    SelectNode {
        node: GraphNodeKey,
    },
    SelectEdge {
        edge: GraphEdgeKey,
    },
    ClearSelection,
    AddNode {
        descriptor_key: String,
    },
    DeleteSelection,
    ConnectPorts {
        from: GraphPortKey,
        to: GraphPortKey,
    },
    DisconnectEdge {
        edge: GraphEdgeKey,
    },
    SetNodeValue {
        node: GraphNodeKey,
        key: String,
        value: String,
    },
    PersistLayout,
    Undo,
    Redo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphShortcutAction {
    AddNode,
    DeleteSelection,
    Undo,
    Redo,
    BuildPreview,
    FocusPreview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphGesturePhase {
    Begin,
    Update,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphGestureIntent {
    PanViewport {
        phase: GraphGesturePhase,
        delta_x: i32,
        delta_y: i32,
    },
    DragNode {
        phase: GraphGesturePhase,
        node: GraphNodeKey,
        delta_x: i32,
        delta_y: i32,
    },
    ConnectPorts {
        phase: GraphGesturePhase,
        from: GraphPortKey,
        hover: Option<GraphPortKey>,
    },
    BoxSelect {
        phase: GraphGesturePhase,
        rect: GraphRect,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphLayoutTransaction {
    pub edit_group: u64,
    pub actions: Vec<GraphEditorAction>,
}

impl GraphLayoutTransaction {
    pub fn new(edit_group: u64, actions: impl IntoIterator<Item = GraphEditorAction>) -> Self {
        Self {
            edit_group,
            actions: actions.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphShortcut {
    pub chord: String,
    pub action: GraphShortcutAction,
}

impl GraphShortcut {
    pub fn new(chord: impl Into<String>, action: GraphShortcutAction) -> Self {
        Self {
            chord: chord.into(),
            action,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_uses_stable_node_keys() {
        let selection = GraphSelection::select_node(GraphNodeKey(7));

        assert!(selection.nodes.contains(&GraphNodeKey(7)));
        assert!(selection.edges.is_empty());
    }

    #[test]
    fn editor_action_is_backend_neutral() {
        let action = GraphEditorAction::ConnectPorts {
            from: GraphPortKey(1),
            to: GraphPortKey(2),
        };

        assert_eq!(
            action,
            GraphEditorAction::ConnectPorts {
                from: GraphPortKey(1),
                to: GraphPortKey(2)
            }
        );
    }

    #[test]
    fn hit_testing_prefers_ports_over_nodes() {
        let scene = GraphHitTestScene {
            nodes: vec![GraphNodeBounds {
                node: GraphNodeKey(1),
                rect: GraphRect::new(0, 0, 120, 80),
            }],
            ports: vec![GraphPortBounds {
                node: GraphNodeKey(1),
                port: GraphPortKey(9),
                rect: GraphRect::new(8, 8, 12, 12),
            }],
        };

        assert_eq!(
            scene.hit_test(GraphPoint::new(10, 10)),
            GraphHitTarget::Port(GraphPortKey(9))
        );
        assert_eq!(
            scene.hit_test(GraphPoint::new(60, 40)),
            GraphHitTarget::Node(GraphNodeKey(1))
        );
        assert_eq!(
            scene.hit_test(GraphPoint::new(200, 200)),
            GraphHitTarget::Background
        );
    }

    #[test]
    fn layout_transaction_groups_source_edits() {
        let transaction = GraphLayoutTransaction::new(
            55,
            [
                GraphEditorAction::Pan {
                    delta_x: 2,
                    delta_y: -1,
                },
                GraphEditorAction::PersistLayout,
            ],
        );

        assert_eq!(transaction.edit_group, 55);
        assert_eq!(transaction.actions.len(), 2);
    }

    #[test]
    fn gestures_are_backend_neutral_intents() {
        let gesture = GraphGestureIntent::ConnectPorts {
            phase: GraphGesturePhase::Update,
            from: GraphPortKey(1),
            hover: Some(GraphPortKey(2)),
        };

        assert_eq!(
            gesture,
            GraphGestureIntent::ConnectPorts {
                phase: GraphGesturePhase::Update,
                from: GraphPortKey(1),
                hover: Some(GraphPortKey(2))
            }
        );
    }
}
