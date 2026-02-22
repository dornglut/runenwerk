use crate::dag::NodeId;
use crate::node::Node;
use crate::scheduler_core::{DAG, Scheduler};
use anyhow::Result;
use std::collections::HashMap;

/// A builder for creating DAGs and Scheduler instances ergonomically.
pub struct SchedulerBuilder<C> {
    dag: DAG<C>,
    name_map: HashMap<String, NodeId>,
    errors: Vec<String>,
}

impl<C> SchedulerBuilder<C> {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            dag: DAG::new(),
            name_map: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Add a node to the DAG and register it by name.
    pub fn add_node(mut self, name: &str, node: Node<C>) -> Self {
        if self.name_map.contains_key(name) {
            self.errors.push(format!(
                "Duplicate node name '{}' in SchedulerBuilder",
                name
            ));
            return self;
        }
        let id: NodeId = self.dag.add_node(node);
        self.name_map.insert(name.to_string(), id);
        self
    }

    /// Add a dependency edge between two nodes by name.
    pub fn add_edge(mut self, from: &str, to: &str) -> Self {
        let Some(&from_id) = self.name_map.get(from) else {
            self.errors
                .push(format!("Unknown dependency source node '{}'", from));
            return self;
        };
        let Some(&to_id) = self.name_map.get(to) else {
            self.errors
                .push(format!("Unknown dependency target node '{}'", to));
            return self;
        };

        if let Err(err) = self.dag.add_edge(from_id, to_id) {
            self.errors.push(err.to_string());
        }
        self
    }

    /// Convenience: add a node and immediately add edges from dependencies.
    pub fn add_node_with_edges(mut self, name: &str, node: Node<C>, depends_on: &[&str]) -> Self {
        if self.name_map.contains_key(name) {
            self.errors.push(format!(
                "Duplicate node name '{}' in SchedulerBuilder",
                name
            ));
            return self;
        }

        let id: NodeId = self.dag.add_node(node);
        self.name_map.insert(name.to_string(), id);
        for &dep in depends_on {
            if let Some(&dep_id) = self.name_map.get(dep) {
                if let Err(err) = self.dag.add_edge(dep_id, id) {
                    self.errors.push(err.to_string());
                }
            } else {
                self.errors
                    .push(format!("Unknown dependency '{}' for node '{}'", dep, name));
            }
        }
        self
    }

    /// Consume the builder and create a Scheduler.
    pub fn build(self) -> Result<Scheduler<C>> {
        if !self.errors.is_empty() {
            anyhow::bail!("Failed to build scheduler:\n{}", self.errors.join("\n"));
        }

        Ok(Scheduler::new(self.dag))
    }
}
