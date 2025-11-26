use test_config::setup_logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _res = setup_logger().await?;

    tracing::error!("Starting demo app...");
    Ok(())
}
