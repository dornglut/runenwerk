//! File: domain/editor/editor_shell/src/tool_suite/capability.rs
//! Purpose: Host-owned capability policy for clean Workbench compositions.

use std::collections::BTreeSet;

use super::{CommandCapabilityKey, ProductCapabilityKey, ResourceCapabilityKey};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HostCapabilityPolicy {
    allow_all: bool,
    allowed_commands: BTreeSet<CommandCapabilityKey>,
    denied_commands: BTreeSet<CommandCapabilityKey>,
    allowed_products: BTreeSet<ProductCapabilityKey>,
    denied_products: BTreeSet<ProductCapabilityKey>,
    allowed_resources: BTreeSet<ResourceCapabilityKey>,
    denied_resources: BTreeSet<ResourceCapabilityKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeniedHostCapabilityRequirement<'a> {
    Command(&'a CommandCapabilityKey),
    Product(&'a ProductCapabilityKey),
    Resource(&'a ResourceCapabilityKey),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HostCapabilityRequirements {
    commands: BTreeSet<CommandCapabilityKey>,
    products: BTreeSet<ProductCapabilityKey>,
    resources: BTreeSet<ResourceCapabilityKey>,
}

impl HostCapabilityRequirements {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn require_command(mut self, key: CommandCapabilityKey) -> Self {
        self.commands.insert(key);
        self
    }

    pub fn require_product(mut self, key: ProductCapabilityKey) -> Self {
        self.products.insert(key);
        self
    }

    pub fn require_resource(mut self, key: ResourceCapabilityKey) -> Self {
        self.resources.insert(key);
        self
    }

    pub fn denied_by(
        &self,
        policy: &HostCapabilityPolicy,
    ) -> Option<DeniedHostCapabilityRequirement<'_>> {
        self.commands
            .iter()
            .find(|key| !policy.allows_command(key))
            .map(DeniedHostCapabilityRequirement::Command)
            .or_else(|| {
                self.products
                    .iter()
                    .find(|key| !policy.allows_product(key))
                    .map(DeniedHostCapabilityRequirement::Product)
            })
            .or_else(|| {
                self.resources
                    .iter()
                    .find(|key| !policy.allows_resource(key))
                    .map(DeniedHostCapabilityRequirement::Resource)
            })
    }
}

impl HostCapabilityPolicy {
    pub fn deny_all() -> Self {
        Self::default()
    }

    pub fn allow_all() -> Self {
        Self {
            allow_all: true,
            ..Self::default()
        }
    }

    pub fn allow_command(mut self, key: CommandCapabilityKey) -> Self {
        self.allowed_commands.insert(key);
        self
    }

    pub fn deny_command(mut self, key: CommandCapabilityKey) -> Self {
        self.denied_commands.insert(key);
        self
    }

    pub fn allow_product(mut self, key: ProductCapabilityKey) -> Self {
        self.allowed_products.insert(key);
        self
    }

    pub fn deny_product(mut self, key: ProductCapabilityKey) -> Self {
        self.denied_products.insert(key);
        self
    }

    pub fn allow_resource(mut self, key: ResourceCapabilityKey) -> Self {
        self.allowed_resources.insert(key);
        self
    }

    pub fn deny_resource(mut self, key: ResourceCapabilityKey) -> Self {
        self.denied_resources.insert(key);
        self
    }

    pub fn allows_command(&self, key: &CommandCapabilityKey) -> bool {
        !self.denied_commands.contains(key)
            && (self.allow_all || self.allowed_commands.contains(key))
    }

    pub fn allows_product(&self, key: &ProductCapabilityKey) -> bool {
        !self.denied_products.contains(key)
            && (self.allow_all || self.allowed_products.contains(key))
    }

    pub fn allows_resource(&self, key: &ResourceCapabilityKey) -> bool {
        !self.denied_resources.contains(key)
            && (self.allow_all || self.allowed_resources.contains(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constrained_policy_denies_commands_by_default() {
        let key = CommandCapabilityKey::new("runenwerk.material_graph.connect_edge").unwrap();

        assert!(!HostCapabilityPolicy::deny_all().allows_command(&key));
    }

    #[test]
    fn explicit_deny_wins_over_allow_all() {
        let key = ProductCapabilityKey::new("runenwerk.material.preview_product").unwrap();
        let policy = HostCapabilityPolicy::allow_all().deny_product(key.clone());

        assert!(!policy.allows_product(&key));
    }

    #[test]
    fn explicit_allow_enables_single_resource() {
        let key = ResourceCapabilityKey::new("runenwerk.project.assets.read").unwrap();
        let policy = HostCapabilityPolicy::deny_all().allow_resource(key.clone());

        assert!(policy.allows_resource(&key));
    }

    #[test]
    fn requirements_report_denied_command_before_mutation() {
        let key = CommandCapabilityKey::new("runenwerk.surface.session_mutation").unwrap();
        let requirements = HostCapabilityRequirements::new().require_command(key.clone());
        let policy = HostCapabilityPolicy::allow_all().deny_command(key.clone());

        assert_eq!(
            requirements.denied_by(&policy),
            Some(DeniedHostCapabilityRequirement::Command(&key))
        );
    }

    #[test]
    fn requirements_accept_all_allowed_capability_planes() {
        let command = CommandCapabilityKey::new("runenwerk.shell.command").unwrap();
        let product = ProductCapabilityKey::new("runenwerk.material.preview_product").unwrap();
        let resource = ResourceCapabilityKey::new("runenwerk.project.assets.read").unwrap();
        let requirements = HostCapabilityRequirements::new()
            .require_command(command.clone())
            .require_product(product.clone())
            .require_resource(resource.clone());
        let policy = HostCapabilityPolicy::deny_all()
            .allow_command(command)
            .allow_product(product)
            .allow_resource(resource);

        assert_eq!(requirements.denied_by(&policy), None);
    }
}
