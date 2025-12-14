use base_infra::runtimes::SpawnTask;
use tracing::info;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.init();

	let task = async {
		info!("test spawn task with tokio");
	};

	task.spawn_task();

	tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
