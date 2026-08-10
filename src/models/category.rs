use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use blog::TryFromRow;

#[derive(Debug, Serialize, Deserialize, Clone, TryFromRow)]
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