use base_infra::result::{AppResult, any_err};
use sea_orm::prelude::async_trait;
use sea_orm::{DatabaseConnection, sqlx};
use sql_infra::cfgs::sqlite::DbConfig;
use sql_infra::error::DBErr;
use sql_infra::{DatabaseConn, DatabaseTrait, SqlxMigrateTrait};
use tracing::info;

#[tokio::main]
async fn main() -> AppResult<()> {
	let db_cfg = DbConfig::default();
	let mg = SqlxMigrator;
	let db = DatabaseConn::setup(&db_cfg, &mg).await?;
	println!("DatabaseConn: {:?}", db);
	Ok(())
}

pub struct SqlxMigrator;

// let db = <Self as DatabaseTrait<DatabaseConn, DbCfg, Mg>>::connect(cfg).await?;
#[async_trait::async_trait]
impl SqlxMigrateTrait for SqlxMigrator {
	async fn migrate(&self, db: &DatabaseConnection) -> AppResult<()> {
		let pool = db.get_sqlite_connection_pool();

		info!("migrations enabled, running...");
		sqlx::migrate!()
			.run(pool)
			.await
			.map_err(any_err(&DBErr::RunMigrationsErr))?;
		info!("migrations successfully ran");
		Ok(())
	}
}
