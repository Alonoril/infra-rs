use base_infra::gen_impl_code_enum;

gen_impl_code_enum! {
	DBErr {
		InitDbPoolErr = ("DBP001", "error while initializing the database connection pool"),
		RunMigrationsErr = ("DBP002", "error while running database migrations"),
		SqlxTxOpenError = ("DBTX00", "Sqlx transaction open error"),
		SqlxTxCommitError = ("DBTX01", "Sqlx transaction commit error"),
		SqlxError = ("DB0000", "Sqlx error"),
	}
}
