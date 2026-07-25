use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
	pub id: i32,
	pub sort_name: String,
	pub sort_description: String,
	pub sort_type: i16,
	pub priority: Option<i32>,
}
impl TryFrom<&Row> for Sort {
	type Error = tokio_postgres::Error;
	fn try_from(row: &Row) -> Result<Self, Self::Error> {
		Ok(Sort {
			id: row.try_get("id")?,
			sort_name: row.try_get("sort_name")?,
			sort_description: row.try_get("sort_description")?,
			sort_type: row.try_get("sort_type")?,
			priority: row.try_get("priority")?,
		})
	}
}
