use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
impl TryFrom<&Row> for Tag {
	type Error = tokio_postgres::Error;
	fn try_from(row: &Row) -> Result<Self, Self::Error> {
		Ok(Tag{
			id: row.try_get("id")?,
			tag_name: row.try_get("tag_name")?,
			status: row.try_get("status")?,
		})
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRequest {
	pub tag_name: String,
	pub status: i32,
}
