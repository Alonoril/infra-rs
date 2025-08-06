pub mod http;
pub mod result;

lazy_static::lazy_static! {
	pub static ref HTTP_TIMEOUT: u64 = 30;
	pub static ref EXPONENTIAL_SECONDS: &'static [f64] =
		&[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,];
}
