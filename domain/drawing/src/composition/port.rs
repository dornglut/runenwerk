//! File: domain/drawing/src/composition/port.rs
//! Purpose: Drawing-owned composite port semantics.

use graph::{PortDefinition, PortDirection, PortId, PortTypeId};

pub const COLOR_PORT_TYPE: PortTypeId = PortTypeId(1);
pub const ALPHA_PORT_TYPE: PortTypeId = PortTypeId(2);
pub const MASK_PORT_TYPE: PortTypeId = PortTypeId(3);
pub const PAPER_PORT_TYPE: PortTypeId = PortTypeId(4);
pub const MATERIAL_MAP_PORT_TYPE: PortTypeId = PortTypeId(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompositePortSemantic {
    Color,
    Alpha,
    Mask,
    Paper,
    MaterialMap,
}

impl CompositePortSemantic {
    pub const fn port_type(self) -> PortTypeId {
        match self {
            Self::Color => COLOR_PORT_TYPE,
            Self::Alpha => ALPHA_PORT_TYPE,
            Self::Mask => MASK_PORT_TYPE,
            Self::Paper => PAPER_PORT_TYPE,
            Self::MaterialMap => MATERIAL_MAP_PORT_TYPE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositePort {
    pub id: PortId,
    pub name: String,
    pub direction: PortDirection,
    pub semantic: CompositePortSemantic,
}

impl CompositePort {
    pub fn new(
        id: PortId,
        name: impl Into<String>,
        direction: PortDirection,
        semantic: CompositePortSemantic,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            direction,
            semantic,
        }
    }

    pub fn to_port_definition(&self) -> PortDefinition {
        PortDefinition::new(
            self.id,
            self.name.clone(),
            self.direction,
            self.semantic.port_type(),
        )
    }
}
