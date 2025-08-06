// fn truncate(&self, len: usize) -> String;
pub trait TruncateStr {
	fn take_len(&self, len: usize) -> String;
}

impl TruncateStr for &str {
	fn take_len(&self, len: usize) -> String {
		if len >= self.len() {
			return self.to_string();
		}

		self.chars().take(len).collect()
	}
}

impl TruncateStr for String {
	fn take_len(&self, len: usize) -> String {
		let s: &str = &self;
		s.take_len(len)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_truncate() {
		let s = "hello world".to_string();
		assert_eq!(s.take_len(5), "hello");
		assert_eq!(s.take_len(11), "hello world");
	}
	#[test]
	fn test_truncate_ref() {
		let s = "hello world";
		assert_eq!(s.take_len(5), "hello");
		assert_eq!(s.take_len(11), "hello world");
	}
}
