use super::action::{UiAction, UiTypedActionDescriptor};

use ui_hosts::{DomainCommand, HostCommand, HostRouteMapVersion, HostRouteMapping};
use ui_program::RouteCapability;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiHostMutationIntent {
    action: UiTypedActionDescriptor,
    host_command: HostCommand,
    domain_command: Option<DomainCommand>,
    required_capabilities: Vec<RouteCapability>,
}

impl UiHostMutationIntent {
    pub fn new(action: UiTypedActionDescriptor, host_command: HostCommand) -> Self {
        let required_capabilities = vec![action.capability().clone()];
        Self {
            action,
            host_command,
            domain_command: None,
            required_capabilities,
        }
    }

    pub fn action(&self) -> &UiTypedActionDescriptor {
        &self.action
    }

    pub fn host_command(&self) -> &HostCommand {
        &self.host_command
    }

    pub fn domain_command(&self) -> Option<&DomainCommand> {
        self.domain_command.as_ref()
    }

    pub fn required_capabilities(&self) -> &[RouteCapability] {
        &self.required_capabilities
    }

    pub fn with_domain_command(mut self, domain_command: DomainCommand) -> Self {
        self.domain_command = Some(domain_command);
        self
    }

    pub fn with_capability(mut self, capability: RouteCapability) -> Self {
        self.required_capabilities.push(capability);
        self
    }

    pub fn to_host_route_mapping(
        &self,
        route_map_version: HostRouteMapVersion,
    ) -> HostRouteMapping {
        let mut mapping = HostRouteMapping::new(
            self.action.route().clone(),
            self.action.schema_version(),
            route_map_version,
            self.host_command.clone(),
        );

        for capability in &self.required_capabilities {
            mapping = mapping.with_capability(capability.clone());
        }

        if let Some(domain_command) = &self.domain_command {
            mapping = mapping.with_domain_command(domain_command.clone());
        }

        mapping
    }
}

pub trait UiActionHandler<A>
where
    A: UiAction,
{
    fn host_intent(&self, action: &A) -> UiHostMutationIntent;
}
