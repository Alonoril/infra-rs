use crate::error::UtlErr;
use base_infra::nar_err;
use base_infra::result::AppResult;
use bigdecimal::{BigDecimal, ToPrimitive};

pub trait ToFloat {
	fn to_f32(&self) -> AppResult<f32>;
	fn to_f64(&self) -> AppResult<f64>;
}

impl ToFloat for BigDecimal {
	fn to_f32(&self) -> AppResult<f32> {
		self.to_ref()
			.to_f32()
			.ok_or_else(nar_err!(&UtlErr::BigDecToF32))
	}

	fn to_f64(&self) -> AppResult<f64> {
		self.to_ref()
			.to_f64()
			.ok_or_else(nar_err!(&UtlErr::BigDecToF32))
	}
}

#[cfg(test)]
mod tests {
	use super::ToFloat;
	use bigdecimal::BigDecimal;

	#[test]
	fn test_to_float() {
		let big_dec = BigDecimal::from(1);
		let f32 = big_dec.to_f32().unwrap();
		let f64 = big_dec.to_f64().unwrap();
		assert_eq!(f32, 1.0);
		assert_eq!(f64, 1.0);
	}
}
