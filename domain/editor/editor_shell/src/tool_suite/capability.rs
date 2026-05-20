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
}
