use anyhow::Result;

mod rendering;
mod runtime;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--evidence") {
        println!("{}", rendering::production_evidence_report()?.format_text());
        return Ok(());
    }

    let rule = if args.iter().any(|arg| arg == "--species") {
        rendering::BoidsRuleFamily::SpeciesInteraction
    } else {
        rendering::BoidsRuleFamily::Flocking
    };
    runtime::run(rule)
}
