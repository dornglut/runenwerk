use scheduler::builder::SchedulerBuilder;
use scheduler::node::Node;

fn main() -> anyhow::Result<()> {
	// 1️⃣ Build scheduler DAG generically over any context (here we use `()`)
	let mut scheduler = SchedulerBuilder::<()>::new()
		// Input node
		.add_node("Input", Node::new("Input", |_ctx: &mut ()| {
			println!("Input node ran");
			Ok(())
		}))
		// Simulation depends on Input
		.add_node_with_edges("Simulation", Node::new("Simulation", |_ctx| {
			println!("Simulation node ran");
			Ok(())
		}), &["Input"])
		// Physics depends on Simulation
		.add_node_with_edges("Physics", Node::new("Physics", |_ctx| {
			println!("Physics node ran");
			Ok(())
		}), &["Simulation"])
		// AI depends on Simulation
		.add_node_with_edges("AI", Node::new("AI", |_ctx| {
			println!("AI node ran");
			Ok(())
		}), &["Simulation"])
		// ChunkWindow depends on Physics
		.add_node_with_edges("ChunkWindow", Node::new("ChunkWindow", |_ctx| {
			println!("ChunkWindow node ran");
			Ok(())
		}), &["Physics"])
		// GPUCompute depends on ChunkWindow + AI
		.add_node_with_edges("GPUCompute", Node::new("GPUCompute", |_ctx| {
			println!("GPUCompute node ran");
			Ok(())
		}), &["ChunkWindow", "AI"])
		// LODUpdate depends on GPUCompute
		.add_node_with_edges("LODUpdate", Node::new("LODUpdate", |_ctx| {
			println!("LODUpdate node ran");
			Ok(())
		}), &["GPUCompute"])
		// Render depends on GPUCompute
		.add_node_with_edges("Render", Node::new("Render", |_ctx| {
			println!("Render node ran");
			Ok(())
		}), &["GPUCompute"])
		// Network depends on Simulation + AI
		.add_node_with_edges("Network", Node::new("Network", |_ctx| {
			println!("Network node ran");
			Ok(())
		}), &["Simulation", "AI"])
		// Audio depends on Simulation + LODUpdate
		.add_node_with_edges("Audio", Node::new("Audio", |_ctx| {
			println!("Audio node ran");
			Ok(())
		}), &["Simulation", "LODUpdate"])
		.build()?;

	println!("DAG exported to debug/dag.dot");

	// 4️⃣ Run scheduler with empty context
	let mut ctx = ();
	scheduler.run(&mut ctx)?;
	println!("Scheduler run complete.");

	Ok(())
}
