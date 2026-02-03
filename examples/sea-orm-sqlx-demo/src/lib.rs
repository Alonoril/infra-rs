use base_infra::result::{AppResult, any_err};
use sea_orm::DatabaseConnection;
use sea_orm::prelude::async_trait;
use sql_infra::SqlxMigrateTrait;
use sql_infra::error::DBErr;
use tracing::info;

pub struct SqlxMigrator;

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
