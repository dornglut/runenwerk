use tracing_subscriber::FmtSubscriber;

pub fn init_tracing() {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(tracing::Level::DEBUG)
		.finish();
	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}