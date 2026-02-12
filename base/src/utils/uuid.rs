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

	pub fn v4_short(&self) -> String {
		self.v4_simple_str()[..8].to_uppercase()
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
		// Convert UUID into two u64 values
		let (high, low) = my_uuid.as_u64_pair();

		// Print the result
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

	#[test]
	fn test_v4_low_u64() {
		let my_uuid = UID.v4_low_u64();
		println!("{}", my_uuid);
	}

	#[test]
	fn test_v4_short() {
		let my_uuid = UID.v4_short();
		println!("{}", my_uuid);
	}
}
