use crate::result::AppError;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum TaskStatus {
	Ok,
	Err(AppError),
}
impl Display for TaskStatus {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			TaskStatus::Ok => write!(f, "success"),
			TaskStatus::Err(err) => write!(f, "app error {err}"),
		}
	}
}
