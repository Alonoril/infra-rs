use std::collections::HashSet;
use std::hash::Hash;

pub trait DiffTrait<T: Eq + PartialEq + Hash> {
	type Output;
	fn diff(self, other: Self) -> Self::Output;
}

impl<T: Eq + PartialEq + Hash> DiffTrait<T> for Vec<T> {
	type Output = Vec<T>;

	fn diff(self, other: Self) -> Self::Output {
		if other.is_empty() {
			return self;
		}

		let others: HashSet<_> = other.into_iter().collect();
		self.into_iter()
			.filter(|item| !others.contains(item))
			.collect()
	}
}

pub trait DedupTrait<T: Eq + PartialEq + Hash + Clone> {
	fn dedup_unordered(self) -> Vec<T>;

	fn dedup_ordered(&mut self);
}

impl<T: Eq + PartialEq + Hash + Clone> DedupTrait<T> for Vec<T> {
	fn dedup_unordered(self) -> Vec<T> {
		let set: HashSet<_> = self.into_iter().collect();
		set.into_iter().collect()
	}

	fn dedup_ordered(&mut self) {
		let mut seen = HashSet::new();
		self.retain(|this| seen.insert(this.clone()));
	}
}

#[cfg(test)]
mod tests {
	use crate::utils::vec_util::{DedupTrait, DiffTrait};
	#[test]
	fn test_diff() {
		let v1 = vec![
			"apple".to_string(),
			"banana".to_string(),
			"orange".to_string(),
		];
		let v2 = vec!["banana".to_string(), "grape".to_string()];

		let diff = v1.diff(v2);
		assert_eq!(diff, vec!["apple".to_string(), "orange".to_string()]);
	}

	#[test]
	fn test_dedup_unordered() {
		let v1 = vec![
			"apple".to_string(),
			"banana".to_string(),
			"orange".to_string(),
			"banana".to_string(),
		];

		let dedup = v1.dedup_unordered();
		println!("dedup: {:?}", dedup);
		assert_eq!(dedup.len(), 3);
	}

	#[test]
	fn test_dedup_ordered() {
		let mut v1 = vec![
			"apple".to_string(),
			"banana".to_string(),
			"orange".to_string(),
			"banana".to_string(),
		];

		v1.dedup_ordered();
		assert_eq!(
			v1,
			vec![
				"apple".to_string(),
				"banana".to_string(),
				"orange".to_string()
			]
		);
	}
}
