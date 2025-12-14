use serde::{Deserialize, Serialize};
use sql_infra::sea_ext::page::{PageQuery, PageSizeTrait};
#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

pub trait ToPagination
where
	Self: PageSizeTrait,
{
	fn to_pagination(&self) -> Pagination;

	fn to_page_query(&self) -> PageQuery;
}

/// Pagination response
#[cfg(feature = "utoipa")]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PageResp<T: ToSchema> {
	/// Paged data list
	pub list: Vec<T>,
	/// Pagination info
	pub pagination: Pagination,
}
#[cfg(not(feature = "utoipa"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct PageResp<T> {
	/// Paged data list
	pub list: Vec<T>,
	/// Pagination info
	pub pagination: Pagination,
}

#[cfg(feature = "utoipa")]
impl<T: ToSchema> PageResp<T> {
	pub fn new(list: Vec<T>, pagination: Pagination) -> Self {
		Self { list, pagination }
	}

	pub fn new_with_page(list: Vec<T>, page: PageQuery) -> Self {
		Self {
			list,
			pagination: page.into(),
		}
	}
}
#[cfg(not(feature = "utoipa"))]
impl<T> PageResp<T> {
	pub fn new(list: Vec<T>, pagination: Pagination) -> Self {
		Self { list, pagination }
	}

	pub fn new_with_page(list: Vec<T>, page: PageQuery) -> Self {
		Self {
			list,
			pagination: page.into(),
		}
	}
}

/// API pagination query
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageParams {
	/// Page number, starting from 1
	#[serde(default = "default_page")]
	pub page: u32,
	/// Page size
	#[serde(default = "default_page_size")]
	pub page_size: u32,
}

fn default_page() -> u32 {
	1
}

fn default_page_size() -> u32 {
	20
}

/// Pagination info
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
	/// Current page number
	pub page: u64,
	/// Page size
	pub page_size: u64,
	/// Total record count
	pub total: u64,
	/// Total page count
	pub total_pages: u64,
}
impl Pagination {
	pub fn new(page: u64, page_size: u64, total: u64, total_pages: u64) -> Self {
		Self {
			page,
			page_size,
			total,
			total_pages,
		}
	}
}

#[cfg(feature = "utoipa")]
impl<T: ToSchema> Default for PageResp<T> {
	fn default() -> Self {
		Self::new(vec![], Pagination::default())
	}
}

#[cfg(not(feature = "utoipa"))]
impl<T> Default for PageResp<T> {
	fn default() -> Self {
		Self::new(vec![], Pagination::default())
	}
}

impl Default for Pagination {
	fn default() -> Self {
		Self {
			page: 1,
			page_size: 20,
			total: 0,
			total_pages: 0,
		}
	}
}

impl From<PageQuery> for Pagination {
	fn from(v: PageQuery) -> Self {
		Self {
			page: v.page,
			page_size: v.page_size,
			total: v.total,
			total_pages: v.total_pages,
		}
	}
}

impl From<Pagination> for PageQuery {
	fn from(v: Pagination) -> Self {
		Self {
			page: v.page,
			page_size: v.page_size,
			total: v.total,
			total_pages: v.total_pages,
		}
	}
}
