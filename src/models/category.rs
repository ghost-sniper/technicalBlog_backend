use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
	pub id: i64,
	pub name: String,
	pub description: String,
	pub is_deleted: i16,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}
impl Category {
	pub fn new(
		id: i64,
		name: String,
		description: String,
		is_deleted: i16,
		created_at: NaiveDateTime,
		updated_at: NaiveDateTime,
	) -> Self {
		Self {
			id,
			name,
			description,
			is_deleted,
			created_at,
			updated_at,
		}
	}
}
