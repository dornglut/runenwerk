fn main() {
    let _trace_guard = engine::utils::setup_tracing();
    eprintln!("runenwerk_draw starting (logs: logs/engine.log)");
    runenwerk_draw::runtime::run().expect("runenwerk draw runtime should start");
}
