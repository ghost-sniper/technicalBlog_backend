use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
	pub id: i64,
	pub name: String,
	pub description: String,
	pub is_deleted: i16,
	pub create_at: NaiveDateTime,
	pub update_at: NaiveDateTime,
}
impl Category {
	pub fn new(
		id: i64,
		name: String,
		description: String,
		is_deleted: i16,
		create_at: NaiveDateTime,
		update_at: NaiveDateTime,
	) -> Self {
		Self {
			id,
			name,
			description,
			is_deleted,
			create_at,
			update_at,
		}
	}
}


impl TryFrom<&Row> for Category {
	type Error = tokio_postgres::Error;

	fn try_from(row: &Row) -> Result<Self, Self::Error> {
		Ok(Category{
			id: row.try_get("id")?,
			name: row.try_get("name")?,
			description: row.try_get("description")?,
			is_deleted: row.try_get("is_deleted")?,
			create_at: row.try_get("create_at")?,
			update_at: row.try_get("update_at")?,
		})
	}
}