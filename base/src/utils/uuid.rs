use uuid::Uuid;
use uuid::fmt::Simple;
pub struct UID;
impl UID {
	pub fn v4(&self) -> Uuid {
		Uuid::new_v4()
	}
	pub fn v4_simple(&self) -> Simple {
		self.v4().simple()
	}

	pub fn v4_simple_str(&self) -> String {
		self.v4_simple().to_string()
	}

	pub fn v4_low_u64(&self) -> u64 {
		let (_, low) = self.v4().as_u64_pair();
		low
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_v4() {
		let my_uuid = UID.v4();
		println!("{:?}", my_uuid);
		// 将UUID转换为两个u64
		let (high, low) = my_uuid.as_u64_pair();

		// 打印结果
		println!("High u64: {}", high);
		println!("Low u64: {}", low);
	}

	#[test]
	fn test_v4_simple() {
		let my_uuid = UID.v4_simple();
		println!("{:?}", my_uuid);
	}

	#[test]
	fn test_v4_simple_str() {
		let my_uuid = UID.v4_simple_str();
		println!("{}", my_uuid);
	}
}
