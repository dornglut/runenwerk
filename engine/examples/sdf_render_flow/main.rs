use anyhow::Result;

mod rendering;
mod runtime;

fn main() -> Result<()> {
    if std::env::args().any(|arg| arg == "--evidence") {
        println!("{}", rendering::production_evidence_report()?.format_text());
        return Ok(());
    }

    runtime::run()
}
