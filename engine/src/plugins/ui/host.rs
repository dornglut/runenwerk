use super::action::{UiAction, UiTypedActionDescriptor};

use ui_hosts::{DomainCommand, HostCommand, HostRouteMapVersion, HostRouteMapping};
use ui_program::{RouteCapability, UiEventPacket};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiHostMutationFailureReason {
    MissingHostData,
    RejectedByHost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiHostMutationRejection {
    failure_reason: UiHostMutationFailureReason,
}

impl UiHostMutationRejection {
    pub fn new(failure_reason: UiHostMutationFailureReason) -> Self {
        Self { failure_reason }
    }

    pub fn missing_host_data() -> Self {
        Self::new(UiHostMutationFailureReason::MissingHostData)
    }

    pub fn rejected_by_host() -> Self {
        Self::new(UiHostMutationFailureReason::RejectedByHost)
    }

    pub fn failure_reason(&self) -> UiHostMutationFailureReason {
        self.failure_reason
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiHostMutationReceipt {
    host_command: HostCommand,
    domain_command: Option<DomainCommand>,
}

impl UiHostMutationReceipt {
    pub fn from_intent(intent: &UiHostMutationIntent) -> Self {
        Self {
            host_command: intent.host_command().clone(),
            domain_command: intent.domain_command().cloned(),
        }
    }

    pub fn host_command(&self) -> &HostCommand {
        &self.host_command
    }

    pub fn domain_command(&self) -> Option<&DomainCommand> {
        self.domain_command.as_ref()
    }
}

pub trait UiHostActionExecutor {
    fn apply(
        &mut self,
        intent: &UiHostMutationIntent,
        packet: &UiEventPacket,
        mapping: &HostRouteMapping,
    ) -> Result<UiHostMutationReceipt, UiHostMutationRejection>;
}
