use anyhow::Result;
use ui_counter_runtime::{CounterRuntimeCliOptions, run_counter_runtime};

fn main() -> Result<()> {
    run_counter_runtime(CounterRuntimeCliOptions::from_env()?)
}
