use anyhow::anyhow;
use base_infra::gen_impl_code_enum;
use base_infra::result::AppError;
use thiserror::Error;

gen_impl_code_enum! {
	RksErr {
		RksDbErr = ("RksDb01", "RksDB error"),
		BcsErr = ("bcs001", "BCS error"),
	}
}

/// This enum defines errors commonly used among `RocksDB` APIs.
#[derive(Debug, Error)]
pub enum RksDbError {
	/// A requested item is not found.
	#[error("{0} not found.")]
	NotFound(String),
	/// Requested too many items.
	#[error("Too many items requested: at least {0} requested, max is {1}")]
	TooManyRequested(u64, u64),
	#[error("Missing state root node at version {0}, probably pruned.")]
	MissingRootError(u64),
	/// Other non-classified error.
	#[error("Other Error: {0}")]
	Other(String),
	#[error("RocksDbIncompleteResult Error: {0}")]
	RocksDbIncompleteResult(String),
	#[error("Other RocksDB Error: {0}")]
	OtherRocksDbError(String),
}

impl From<anyhow::Error> for RksDbError {
	fn from(error: anyhow::Error) -> Self {
		Self::Other(format!("{}", error))
	}
}

impl From<rocksdb::Error> for RksDbError {
	fn from(error: rocksdb::Error) -> Self {
		Self::OtherRocksDbError(format!("{}", error))
	}
}

impl From<std::num::ParseIntError> for RksDbError {
	fn from(error: std::num::ParseIntError) -> Self {
		Self::Other(format!("{}", error))
	}
}

impl From<RksDbError> for AppError {
	fn from(err: RksDbError) -> Self {
		AppError::Anyhow(&RksErr::RksDbErr, anyhow!(err))
	}
}

#[derive(Debug, Error)]
pub enum CodecError {
	#[error(transparent)]
	BcsErr(#[from] bcs::Error),
}

impl From<CodecError> for AppError {
	fn from(err: CodecError) -> Self {
		AppError::Anyhow(&RksErr::BcsErr, anyhow!(err))
	}
}
