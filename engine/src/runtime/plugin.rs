use super::EngineData;
use anyhow::Result;
use scheduler::{Node, Scheduler, SchedulerBuilder};

type EngineSystemFn = Box<dyn FnMut(&mut EngineData) -> Result<()> + Send>;

struct ScheduledNode {
    name: String,
    depends_on: Vec<String>,
    system: EngineSystemFn,
}

struct ScheduledEdge {
    from: String,
    to: String,
}

pub struct EngineScheduleBuilder {
    nodes: Vec<ScheduledNode>,
    edges: Vec<ScheduledEdge>,
}

impl EngineScheduleBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node<F>(&mut self, name: impl Into<String>, system: F) -> &mut Self
    where
        F: FnMut(&mut EngineData) -> Result<()> + Send + 'static,
    {
        self.nodes.push(ScheduledNode {
            name: name.into(),
            depends_on: Vec::new(),
            system: Box::new(system),
        });
        self
    }

    pub fn add_node_with_edges<F>(
        &mut self,
        name: impl Into<String>,
        system: F,
        depends_on: &[&str],
    ) -> &mut Self
    where
        F: FnMut(&mut EngineData) -> Result<()> + Send + 'static,
    {
        self.nodes.push(ScheduledNode {
            name: name.into(),
            depends_on: depends_on.iter().map(|value| value.to_string()).collect(),
            system: Box::new(system),
        });
        self
    }

    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>) -> &mut Self {
        self.edges.push(ScheduledEdge {
            from: from.into(),
            to: to.into(),
        });
        self
    }

    pub fn build_scheduler(self) -> Result<Scheduler<EngineData>> {
        let Self { nodes, edges } = self;
        let mut builder = SchedulerBuilder::<EngineData>::new();
        let mut implicit_edges = Vec::new();

        for node in nodes {
            let node_name = node.name.clone();
            for dep in &node.depends_on {
                implicit_edges.push(ScheduledEdge {
                    from: dep.clone(),
                    to: node_name.clone(),
                });
            }
            let scheduler_node = Node {
                name: node_name.clone(),
                func: node.system,
            };
            builder = builder.add_node(&node_name, scheduler_node);
        }

        for edge in implicit_edges.into_iter().chain(edges) {
            builder = builder.add_edge(&edge.from, &edge.to);
        }
        builder.build()
    }
}

impl Default for EngineScheduleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EnginePlugin: Send + Sync {
    fn name(&self) -> &'static str;

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()>;

    fn setup(&self, _data: &mut EngineData) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::EngineScheduleBuilder;

    #[test]
    fn schedule_builder_collects_nodes_and_dependencies() {
        let mut builder = EngineScheduleBuilder::new();
        builder.add_node("time", |_| Ok(()));
        builder.add_node_with_edges("ui_input", |_| Ok(()), &["time"]);

        assert_eq!(builder.nodes.len(), 2);
        assert_eq!(builder.nodes[0].name, "time");
        assert!(builder.nodes[0].depends_on.is_empty());
        assert_eq!(builder.nodes[1].name, "ui_input");
        assert_eq!(builder.nodes[1].depends_on, vec!["time".to_string()]);
    }

    #[test]
    fn schedule_builder_builds_linear_scheduler() {
        let mut builder = EngineScheduleBuilder::new();
        builder
            .add_node("time", |_| Ok(()))
            .add_node_with_edges("ui_input", |_| Ok(()), &["time"])
            .add_node_with_edges("render", |_| Ok(()), &["ui_input"]);

        assert!(builder.build_scheduler().is_ok());
    }

    #[test]
    fn schedule_builder_rejects_unknown_dependencies() {
        let mut builder = EngineScheduleBuilder::new();
        builder.add_node_with_edges("ui_input", |_| Ok(()), &["time"]);

        let err = match builder.build_scheduler() {
            Ok(_) => panic!("missing dependency should fail schedule build"),
            Err(err) => err.to_string(),
        };
        assert!(err.contains("Unknown dependency source node 'time'"));
    }

    #[test]
    fn schedule_builder_rejects_unknown_manual_edges() {
        let mut builder = EngineScheduleBuilder::new();
        builder.add_node("time", |_| Ok(()));
        builder.add_edge("time", "overlay_ui_layout");

        let err = match builder.build_scheduler() {
            Ok(_) => panic!("unknown target should fail schedule build"),
            Err(err) => err.to_string(),
        };
        assert!(err.contains("Unknown dependency target node 'overlay_ui_layout'"));
    }

    #[test]
    fn schedule_builder_rejects_duplicate_node_names() {
        let mut builder = EngineScheduleBuilder::new();
        builder
            .add_node("time", |_| Ok(()))
            .add_node("time", |_| Ok(()));

        let err = match builder.build_scheduler() {
            Ok(_) => panic!("duplicate node name should fail schedule build"),
            Err(err) => err.to_string(),
        };
        assert!(err.contains("Duplicate node name 'time'"));
    }
}
