use super::PipelineKey;
use anyhow::{Result, bail};
use std::collections::{BTreeSet, VecDeque};

pub type ResourceId = &'static str;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PassKind {
    Compute,
    Render,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PassHandle(pub usize);

#[derive(Debug, Clone)]
pub struct PassNode {
    pub name: &'static str,
    pub kind: PassKind,
    pub pipeline: PipelineKey,
    pub reads: Vec<ResourceId>,
    pub writes: Vec<ResourceId>,
    pub depends_on: Vec<PassHandle>,
}

#[derive(Debug, Default, Clone)]
pub struct FrameGraph {
    nodes: Vec<PassNode>,
}

impl FrameGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    pub fn compute_pass(
        &mut self,
        name: &'static str,
        pipeline: PipelineKey,
    ) -> PassBuilder<'_> {
        PassBuilder::new(self, PassKind::Compute, name, pipeline)
    }

    pub fn render_pass(
        &mut self,
        name: &'static str,
        pipeline: PipelineKey,
    ) -> PassBuilder<'_> {
        PassBuilder::new(self, PassKind::Render, name, pipeline)
    }

    pub fn node(&self, handle: PassHandle) -> Option<&PassNode> {
        self.nodes.get(handle.0)
    }

    pub fn execution_order(&self) -> Result<Vec<PassHandle>> {
        let node_count = self.nodes.len();
        if node_count <= 1 {
            return Ok((0..node_count).map(PassHandle).collect());
        }

        let mut indegree = vec![0usize; node_count];
        let mut edges = vec![Vec::<usize>::new(); node_count];

        for node_idx in 0..node_count {
            let deps = self.resolved_dependencies(node_idx)?;
            indegree[node_idx] = deps.len();
            for dep in deps {
                edges[dep].push(node_idx);
            }
        }

        let mut ready = BTreeSet::<usize>::new();
        for (idx, degree) in indegree.iter().enumerate() {
            if *degree == 0 {
                ready.insert(idx);
            }
        }

        let mut queue = VecDeque::<usize>::new();
        while let Some(first) = ready.pop_first() {
            queue.push_back(first);
        }

        let mut output = Vec::with_capacity(node_count);
        while let Some(node) = queue.pop_front() {
            output.push(PassHandle(node));
            for next in &edges[node] {
                indegree[*next] = indegree[*next].saturating_sub(1);
                if indegree[*next] == 0 {
                    ready.insert(*next);
                }
            }
            while let Some(first) = ready.pop_first() {
                queue.push_back(first);
            }
        }

        if output.len() != node_count {
            let stuck = (0..node_count)
                .filter(|idx| indegree[*idx] > 0)
                .map(|idx| self.nodes[idx].name)
                .collect::<Vec<_>>()
                .join(", ");
            bail!("frame graph contains a dependency cycle: {stuck}");
        }

        Ok(output)
    }

    fn resolved_dependencies(&self, node_idx: usize) -> Result<Vec<usize>> {
        let mut deps = BTreeSet::<usize>::new();
        let Some(node) = self.nodes.get(node_idx) else {
            bail!("pass handle out of bounds");
        };

        for handle in &node.depends_on {
            if handle.0 >= self.nodes.len() {
                bail!(
                    "pass '{}' depends on unknown handle {}",
                    node.name,
                    handle.0
                );
            }
            deps.insert(handle.0);
        }

        for earlier_idx in 0..node_idx {
            let earlier = &self.nodes[earlier_idx];
            let earlier_touches_write = earlier
                .writes
                .iter()
                .any(|res| node.reads.contains(res) || node.writes.contains(res));
            let later_writes_after_read = node
                .writes
                .iter()
                .any(|res| earlier.reads.contains(res) || earlier.writes.contains(res));
            if earlier_touches_write || later_writes_after_read {
                deps.insert(earlier_idx);
            }
        }

        Ok(deps.into_iter().collect())
    }
}

pub struct PassBuilder<'a> {
    graph: &'a mut FrameGraph,
    node: PassNode,
}

impl<'a> PassBuilder<'a> {
    fn new(graph: &'a mut FrameGraph, kind: PassKind, name: &'static str, pipeline: PipelineKey) -> Self {
        Self {
            graph,
            node: PassNode {
                name,
                kind,
                pipeline,
                reads: Vec::new(),
                writes: Vec::new(),
                depends_on: Vec::new(),
            },
        }
    }

    pub fn reads(mut self, resources: &[ResourceId]) -> Self {
        self.node.reads.extend_from_slice(resources);
        self
    }

    pub fn writes(mut self, resources: &[ResourceId]) -> Self {
        self.node.writes.extend_from_slice(resources);
        self
    }

    pub fn depends_on(mut self, dependency: PassHandle) -> Self {
        self.node.depends_on.push(dependency);
        self
    }

    pub fn build(self) -> PassHandle {
        let idx = self.graph.nodes.len();
        self.graph.nodes.push(self.node);
        PassHandle(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameGraph, PassHandle, PipelineKey};

    #[test]
    fn explicit_dependencies_are_honored() {
        let mut graph = FrameGraph::new();
        let first = graph
            .compute_pass("first", PipelineKey::WorldComputeBasic)
            .build();
        let second = graph
            .render_pass("second", PipelineKey::WorldComposeFullscreen)
            .depends_on(first)
            .build();
        let order = graph.execution_order().expect("graph should sort");
        assert_eq!(order, vec![first, second]);
    }

    #[test]
    fn write_then_read_resource_infers_dependency() {
        let mut graph = FrameGraph::new();
        let writer = graph
            .compute_pass("writer", PipelineKey::WorldComputeBasic)
            .writes(&["world_color"])
            .build();
        let reader = graph
            .render_pass("reader", PipelineKey::WorldComposeFullscreen)
            .reads(&["world_color"])
            .build();
        let order = graph.execution_order().expect("graph should sort");
        assert_eq!(order, vec![writer, reader]);
    }

    #[test]
    fn cycle_is_reported() {
        let mut graph = FrameGraph::new();
        let a = graph
            .compute_pass("a", PipelineKey::WorldComputeBasic)
            .build();
        let _b = graph
            .render_pass("b", PipelineKey::WorldComposeFullscreen)
            .depends_on(a)
            .build();
        // Add a backwards explicit dependency to force a cycle.
        if let Some(node) = graph.nodes.get_mut(a.0) {
            node.depends_on.push(PassHandle(1));
        }
        let err = graph
            .execution_order()
            .expect_err("cycle should produce an error");
        assert!(err.to_string().contains("cycle"));
    }
}
