use serde::{Deserialize, Serialize};

pub trait PageSizeTrait {
	fn page(&self) -> u64;
	fn page_size(&self) -> u64;
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct PageQuery {
	pub page: u64,
	pub page_size: u64,
	pub total: u64,
	pub total_pages: u64,
}

impl PageQuery {
	pub fn new(page: u64, page_size: u64, total: u64) -> Self {
		let total_pages = if total % page_size == 0 {
			total / page_size
		} else {
			total / page_size + 1
		};
		Self {
			page,
			page_size,
			total,
			total_pages,
		}
	}

	pub fn with_total(self, total: u64) -> Self {
		let total_pages = if total % self.page_size == 0 {
			total / self.page_size
		} else {
			total / self.page_size + 1
		};

		Self {
			total,
			total_pages,
			..self
		}
	}
}

impl Default for PageQuery {
	fn default() -> Self {
		Self {
			page: 1,
			page_size: 10,
			total: 0,
			total_pages: 0,
		}
	}
}

#[derive(Debug)]
pub struct SqlPageResp<T> {
	pub list: Vec<T>,
	pub page: PageQuery,
}

impl<T> SqlPageResp<T> {
	pub fn new(list: Vec<T>, page: PageQuery) -> Self {
		Self { list, page }
	}
}

impl Default for SqlPageResp<()> {
	fn default() -> Self {
		Self {
			list: vec![],
			page: PageQuery::default(),
		}
	}
}
