use runen_ecs::SystemSet;

#[derive(Debug, Copy, Clone, PartialEq, Eq, SystemSet)]
pub enum NetPreUpdateSet {
    Receive,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, SystemSet)]
pub enum NetFixedSet {
    Prediction,
    Replication,
}
