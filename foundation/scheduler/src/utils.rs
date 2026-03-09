use crate::dag::DAG;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Write the DAG to a DOT file in the given folder
pub fn export_dag_dot<C>(dag: &DAG<C>, folder: &str, filename: &str) -> Result<()> {
    // Ensure folder exists
    fs::create_dir_all(folder)?;

    let path = Path::new(folder).join(filename);
    let dot = dag.to_dot();
    fs::write(&path, dot)?;

    println!("DAG exported to {:?}", path);
    Ok(())
}
