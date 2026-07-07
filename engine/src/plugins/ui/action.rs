use super::screen::{UiTypedIdentityError, validate_typed_contract_id};

use ui_program::{RouteCapability, RouteId, RouteSchemaVersion};
use ui_schema::UiSchemaRef;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiTypedActionId(String);

impl UiTypedActionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("typed UI action IDs must be stable namespaced IDs")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, UiTypedIdentityError> {
        Ok(Self(validate_typed_contract_id("action", value.into())?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiTypedActionDescriptor {
    action_id: UiTypedActionId,
    route: RouteId,
    schema_version: RouteSchemaVersion,
    payload_schema: UiSchemaRef,
    capability: RouteCapability,
}

impl UiTypedActionDescriptor {
    pub fn new(
        action_id: UiTypedActionId,
        route: RouteId,
        schema_version: RouteSchemaVersion,
        payload_schema: UiSchemaRef,
        capability: RouteCapability,
    ) -> Self {
        Self {
            action_id,
            route,
            schema_version,
            payload_schema,
            capability,
        }
    }

    pub fn action_id(&self) -> &UiTypedActionId {
        &self.action_id
    }

    pub fn route(&self) -> &RouteId {
        &self.route
    }

    pub fn schema_version(&self) -> RouteSchemaVersion {
        self.schema_version
    }

    pub fn payload_schema(&self) -> &UiSchemaRef {
        &self.payload_schema
    }

    pub fn capability(&self) -> &RouteCapability {
        &self.capability
    }
}

pub trait UiAction {
    fn action_descriptor(&self) -> UiTypedActionDescriptor;
}
