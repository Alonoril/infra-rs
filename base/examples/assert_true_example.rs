use base_infra::assert_true;
use base_infra::result::{AppResult, SysErr};

fn main() -> AppResult<()> {
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.init();

	assert_true!(true, &SysErr::InvalidParams, "has no EDS token");

	Ok(())
}
