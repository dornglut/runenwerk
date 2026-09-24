use anyhow::Result;

mod rendering;
mod runtime;

fn main() -> Result<()> {
    runtime::run()
}
