use base_infra::result::AppResult;
use sea_orm_sqlx_demo::SqlxMigrator;
use sql_infra::cfgs::sqlite::DbConfig;
use sql_infra::{DatabaseConn, DatabaseTrait};

#[tokio::main]
async fn main() -> AppResult<()> {
	let db_cfg = DbConfig::default();
	let mg = SqlxMigrator;
	let db = DatabaseConn::setup(&db_cfg, &mg).await?;
	println!("DatabaseConn: {:?}", db);
	Ok(())
}
