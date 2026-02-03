use crate::error::DBErr;
use crate::{DatabaseConn, ServerVersion};
use base_infra::result::AppResult;
use base_infra::{map_err, nar_err};
use sea_orm::{ConnectionTrait, DbBackend, Statement};

#[async_trait::async_trait]
impl ServerVersion for DatabaseConn {
	async fn version(&self) -> AppResult<String> {
		let stmt = Statement::from_string(DbBackend::Postgres, "SELECT version()");
		let row = self.pool.query_one(stmt).await;
		let row = row
			.map_err(map_err!(&DBErr::GetVersion))?
			.ok_or_else(nar_err!(&DBErr::VersionNotFound))?;
		row.try_get("", "version")
			.map_err(map_err!(&DBErr::TryGetVersion))
	}
}
