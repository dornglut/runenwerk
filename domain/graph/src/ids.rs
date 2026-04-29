//! File: domain/graph/src/ids.rs
//! Purpose: Stable typed identifiers for graph definitions.

macro_rules! graph_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            pub const fn raw(self) -> u64 {
                self.0
            }
        }
    };
}

graph_id!(GraphId);
graph_id!(NodeId);
graph_id!(PortId);
graph_id!(EdgeId);
graph_id!(PortTypeId);
