use blog::TryFromRow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Debug, Clone, Serialize, Deserialize, TryFromRow)]
pub struct Session {
	pub id: i64,
	pub user_id: i32,
	pub token: String,
	pub payload: Option<serde_json::Value>,
	pub create_time: DateTime<Utc>,
	pub expires_at: DateTime<Utc>,
	pub last_used_at: Option<DateTime<Utc>>,
}
