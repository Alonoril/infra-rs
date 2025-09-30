use base_infra::result::{AppResult, SysErr};
use base_infra::{err, map_err};
use std::io::{Error, ErrorKind};

fn main() -> AppResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // return_io_err().map_err(map_err!(&SysErr::InternalError))?;
    let msg = "sdt::io error".into();
    return_io_err().map_err(map_err!(&SysErr::InternalError, msg))?;
    Ok(())
}

fn return_io_err() -> Result<(), std::io::Error> {
    // Err(Error::new(ErrorKind::Other, "Some error"))
    Err(Error::from(ErrorKind::UnexpectedEof))
}

fn ret_ext_macro_err() -> AppResult<()> {
    err!(&SysErr::SystemError, "some error")
}

fn ret_macro_err() -> AppResult<()> {
    err!(&SysErr::SystemError)
}
