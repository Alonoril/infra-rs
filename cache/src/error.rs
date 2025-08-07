use crate::schema::CacheTtl;
use base_infra::gen_impl_code_enum;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BaseError {
    #[error("{0}")]
    With(String),
    #[error("failed to bind TcpListener: {0}")]
    IoError(#[from] io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("cache not initialized for ttl: {0:?}")]
    CacheNotInit(CacheTtl),
}

gen_impl_code_enum! {
    CacheErr {
        CacheNotInit = ("Cache1", "cache not initialized for ttl"),
    }
}
