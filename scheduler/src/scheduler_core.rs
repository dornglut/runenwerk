pub(crate) use crate::dag::DAG;
use crate::{Node, NodeId};
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::time::Instant;
use tracing::{Level, info, span};

pub struct Scheduler<C> {
    dag: DAG<C>,
    execution_order: Vec<NodeId>,
    dirty: bool,
}

impl<C> Scheduler<C> {
    pub fn new(dag: DAG<C>) -> Self {
        Self {
            dag,
            execution_order: vec![],
            dirty: true,
        }
    }

    pub fn run(&mut self, ctx: &mut C) -> Result<()> {
        if self.dirty {
            self.rebuild_execution_order()
                .context("Failed to rebuild execution order")?;
        }

        let run_span = span!(Level::INFO, "scheduler_run");
        let _enter = run_span.enter();

        for node_id in &self.execution_order {
            if let Some(node) = self.dag.get_node_mut(*node_id) {
                let node_span =
                    span!(Level::DEBUG, "node_run", node_id = ?node_id, node_name = %node.name);
                let _enter_node = node_span.enter();

                let start = Instant::now();
                (node.func)(ctx).with_context(|| format!("Failed to run node {:?}", node_id))?;
                let duration = start.elapsed();

                if duration.as_millis() > 1 {
                    info!(
                        "Node {:?} ({}) took {} ms",
                        node_id,
                        node.name,
                        duration.as_millis()
                    );
                }
            }
        }
        Ok(())
    }

    pub fn rebuild_execution_order(&mut self) -> Result<()> {
        use std::collections::HashSet;

        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut order = Vec::new();

        fn visit<C>(
            dag: &DAG<C>,
            node: NodeId,
            visited: &mut HashSet<NodeId>,
            visiting: &mut HashSet<NodeId>,
            order: &mut Vec<NodeId>,
        ) -> Result<()> {
            if visited.contains(&node) {
                return Ok(());
            }

            if !visiting.insert(node) {
                anyhow::bail!(
                    "Cycle detected involving node {:?} ({})",
                    node,
                    dag.get_node(node)
                        .map(|n| &n.name)
                        .unwrap_or(&"<unknown>".to_string())
                );
            }

            if let Some(edges) = dag.edges.get(&node) {
                for &next in edges {
                    visit(dag, next, visited, visiting, order)?;
                }
            }

            visiting.remove(&node);
            visited.insert(node);
            order.push(node);
            Ok(())
        }

        let mut node_ids: Vec<_> = self.dag.nodes.keys().copied().collect();
        node_ids.sort_by_key(|id| id.0);
        for node_id in node_ids {
            visit(&self.dag, node_id, &mut visited, &mut visiting, &mut order)?;
        }

        order.reverse();
        self.execution_order = order;
        self.dirty = false;
        Ok(())
    }

    // --- Controlled DAG mutation APIs ---
    pub fn add_node(&mut self, node: Node<C>) -> NodeId {
        let id = self.dag.add_node(node);
        self.dirty = true;
        id
    }

    pub fn remove_node(&mut self, id: NodeId) {
        self.dag.remove_node(id);
        self.dirty = true;
    }

    pub fn add_dependency(&mut self, before: NodeId, after: NodeId) -> Result<()> {
        self.dag.add_edge(before, after)?;
        self.dirty = true;
        Ok(())
    }

    pub fn remove_dependency(&mut self, before: NodeId, after: NodeId) {
        self.dag.remove_edge(before, after);
        self.dirty = true;
    }

    /// Export the scheduler's DAG to a DOT file
    pub fn export_dot(&self, folder: &str, filename: &str) -> Result<()> {
        fs::create_dir_all(folder)?;
        let path = Path::new(folder).join(filename);
        let dot = self.dag.to_dot();
        fs::write(&path, dot)?;

        info!("DAG exported to {:?}", path);
        Ok(())
    }
}
