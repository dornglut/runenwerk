use std::collections::{HashMap, HashSet};
use crate::node::Node;
use anyhow::Result;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub u64);

pub struct DAG<C> {
	pub(crate) nodes: HashMap<NodeId, Node<C>>,
	pub(crate) edges: HashMap<NodeId, Vec<NodeId>>,
	next_id: u64,
}

impl<C> DAG<C> {
	pub fn new() -> Self {
		Self {
			nodes: HashMap::new(),
			edges: HashMap::new(),
			next_id: 0,
		}
	}

	fn generate_id(&mut self) -> NodeId {
		let id = NodeId(self.next_id);
		self.next_id += 1;
		id
	}

	pub fn add_node(&mut self, node: Node<C>) -> NodeId {
		let id = self.generate_id();
		self.nodes.insert(id, node);
		self.edges.entry(id).or_default();
		id
	}
	pub fn remove_node(&mut self, id: NodeId) {
		self.nodes.remove(&id);
		self.edges.remove(&id);
		for edges in self.edges.values_mut() {
			edges.retain(|&nid| nid != id);
		}
	}
	pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> Result<()> {
		if from == to {
			anyhow::bail!("Self dependency is not allowed for node {:?}", from);
		}
		if !self.nodes.contains_key(&from) {
			anyhow::bail!("Source node {:?} does not exist", from);
		}
		if !self.nodes.contains_key(&to) {
			anyhow::bail!("Target node {:?} does not exist", to);
		}

		let outgoing = self.edges.entry(from).or_default();
		if !outgoing.contains(&to) {
			outgoing.push(to);
		}
		Ok(())
	}
	pub fn remove_edge(&mut self, from: NodeId, to: NodeId) {
		if let Some(edges) = self.edges.get_mut(&from) {
			edges.retain(|&nid| nid != to);
		}
	}
	pub fn get_node(&self, id: NodeId) -> Option<&Node<C>> {
		self.nodes.get(&id)
	}
	pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node<C>> {
		self.nodes.get_mut(&id)
	}
	pub fn topological_sort(&self) -> Vec<NodeId> {
		let mut visited = HashSet::new();
		let mut stack = Vec::new();

		fn visit(
			node_id: NodeId,
			edges: &HashMap<NodeId, Vec<NodeId>>,
			visited: &mut HashSet<NodeId>,
			stack: &mut Vec<NodeId>,
		) {
			if visited.contains(&node_id) { return; }
			visited.insert(node_id);

			if let Some(neighbours) = edges.get(&node_id) {
				for &neighbour in neighbours {
					visit(neighbour, edges, visited, stack);
				}
			}

			stack.push(node_id);
		}

		for &node_id in self.nodes.keys() {
			visit(node_id, &self.edges, &mut visited, &mut stack);
		}

		stack.reverse();
		stack
	}
}

impl<C> DAG<C> {
	/// Print a textual representation of the DAG (nodes + edges)
	pub fn print(&self) {
		println!("DAG Nodes:");
		for (id, node) in &self.nodes {
			println!("  Node {:?}: {}", id.0, node.name);
		}

		println!("DAG Edges:");
		for (from, to_list) in &self.edges {
			for &to in to_list {
				println!("  {:?} -> {:?}", from.0, to.0);
			}
		}
	}

	/// Optional: return a Graphviz `.dot` string for visualization
	pub fn to_dot(&self) -> String {
		let mut dot = String::from("digraph DAG {\n");
		for (id, node) in &self.nodes {
			dot.push_str(&format!("  {} [label=\"{}\"];\n", id.0, node.name));
		}
		for (from, to_list) in &self.edges {
			for &to in to_list {
				dot.push_str(&format!("  {} -> {};\n", from.0, to.0));
			}
		}
		dot.push_str("}\n");
		dot
	}
}
