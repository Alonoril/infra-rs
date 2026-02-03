use base_infra::result::AppResult;
use sea_orm_sqlx_demo::SqlxMigrator;
use sql_infra::{DatabaseConn, DatabaseTrait, ServerVersion};

#[tokio::main]
async fn main() -> AppResult<()> {
	let (cfg, _guard) = test_config::setup_logger().await?;
	let db = DatabaseConn::setup(&cfg.db_config, &SqlxMigrator).await?;
	println!("DatabaseConn: {:?}", db);

	let version = db.version().await?;
	println!("ServerVersion: {version}");
	Ok(())
}
