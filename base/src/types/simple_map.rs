#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SimpleMap<K, V> {
	data: Vec<Element<K, V>>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Element<K, V> {
	key: K,
	value: V,
}

impl<K, V> SimpleMap<K, V> {
	pub fn new() -> Self {
		Self { data: vec![] }
	}

	pub fn into_pairs(self) -> Vec<(K, V)> {
		self.data.into_iter().map(|e| (e.key, e.value)).collect()
	}
}

impl<K, V> Default for SimpleMap<K, V> {
	fn default() -> Self {
		Self::new()
	}
}
