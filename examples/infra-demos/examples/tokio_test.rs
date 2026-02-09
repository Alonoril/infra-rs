use base_infra::result::AppResult;
use base_infra::runtimes::Tokio;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> AppResult<()> {
    let (cfg, _guard) = test_config::setup_logger().await?;
    info!("starting server...");

    // spawn_to_main_thread().await;

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    Ok(())
}

// async fn spawn_to_main_thread() {
// 	let task = async {
// 		warn!("test spawn task to main thread");
// 		panic!("test panic")
// 	};
//
// 	Tokio.spawn_sys_thread(task);
// }
