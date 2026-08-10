use blog::TryFromRow;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Debug, Clone, Serialize, Deserialize, TryFromRow)]
pub struct Sort {
	pub id: i32,
	pub sort_name: String,
	pub sort_description: String,
	pub sort_type: i16,
	pub priority: Option<i32>,
}
