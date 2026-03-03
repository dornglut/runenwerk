use crate::Entity;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EntityError {
    #[error("entity {entity:?} does not exist")]
    NoSuchEntity { entity: Entity },
    #[error("entity {entity:?} is missing component {component}")]
    MissingComponent {
        entity: Entity,
        component: &'static str,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ResourceError {
    #[error("resource {resource} does not exist")]
    Missing { resource: &'static str },
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error(transparent)]
    Entity(#[from] EntityError),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum QueryError {
    #[error("query expected exactly one result but found none")]
    NoResults,
    #[error("query expected exactly one result but found {count}")]
    MultipleResults { count: usize },
}
