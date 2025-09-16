use crate::result::AppResult;

/// Usage
///
/// assert_true!(self.module_name, err!(&BcErr::InvalidParams, "module_name is empty"));
#[macro_export]
macro_rules! assert_true {
	($cond:expr, $err_code:expr) => {
		if $cond {
			return $crate::err!($err_code);
		}
	};

	($cond:expr,$code:expr, $err:expr) => {
		if $cond {
			return $crate::err!($code, $err);
		}
	};
}

pub trait Checker {
	fn check(&self) -> AppResult<()>;
}

pub trait Validator {
	fn validate(&self) -> AppResult<()>;
}

impl<T> Validator for T
where
	T: Checker,
{
	fn validate(&self) -> AppResult<()> {
		self.check()
	}
}
