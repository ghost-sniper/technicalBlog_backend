use blog::TryFromRow;
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Debug, Clone, Serialize, Deserialize, TryFromRow)]
pub struct UserVo {
	pub id: Option<i64>,
	pub username: String,
	pub phone_number: Option<String>,
	pub email: Option<String>,
	pub password: String,
	pub gender: Option<i16>,
	pub avatar: Option<String>,
	pub user_level: Option<i16>,
	pub introduction: Option<String>,
	pub subscribe: Option<String>,
	pub create_time: Option<DateTime<FixedOffset>>,
	pub update_time: Option<DateTime<FixedOffset>>,
	pub update_user: Option<String>,
	pub is_boss: Option<bool>,
	pub access_token: Option<String>,
	pub code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, TryFromRow)]
pub struct UserLoginRequest {
	pub username: Option<String>,
	pub password: Option<String>,
	pub public_key: Option<String>,
	pub iv: Option<String>,
	pub aes_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TryFromRow)]
pub struct User {
	pub id: i32,
	pub username: String,
	#[serde(skip_serializing)]
	pub password: String,
	pub phone_number: Option<String>,
	pub email: Option<String>,
	pub user_status: i16,
	pub gender: i16,
	pub open_id: Option<String>,
	pub avatar: Option<String>,
	pub admire: Option<String>,
	pub subscribe: Option<String>,
	pub introduction: Option<String>,
	pub user_type: i16,
	pub user_level: i16,
	pub create_time: NaiveDateTime,
	pub update_time: NaiveDateTime,
	pub update_by: Option<String>,
	pub deleted: i16,
}
