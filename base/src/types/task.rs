use std::fmt::{Display, Formatter};
use crate::result::AppError;

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
