use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
	pub id: i64,
	pub user_id: i32,
	pub token: String,
	pub payload: Option<serde_json::Value>,
	pub create_time: DateTime<Utc>,
	pub expires_at: DateTime<Utc>,
	pub last_used_at: Option<DateTime<Utc>>,
}

impl TryFrom<&Row> for Session {
	type Error = tokio_postgres::Error;
	fn try_from(row: &Row) -> Result<Self, Self::Error> {
		Ok(Session {
			id: row.try_get("id")?,
			user_id: row.try_get("user_id")?,
			token: row.try_get("token")?,
			payload: row.try_get("payload")?,
			create_time: row.try_get("create_time")?,
			expires_at: row.try_get("expires_at")?,
			last_used_at: row.try_get("last_used_at")?,
		})
	}
}
