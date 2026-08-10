use blog::TryFromRow;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Serialize, Deserialize, Debug, Clone, TryFromRow)]
pub struct Tag {
	pub id: i64,
	pub tag_name: String,
	pub status: i32,
}
impl Tag {
	pub fn new(id: i64, tag_name: String, status: i32) -> Self {
		Self {
			id,
			tag_name,
			status,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRequest {
	pub tag_name: String,
	pub status: i32,
}
