use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserLoginParams {
	pub account: Option<String>,
	pub password: Option<String>,
	pub is_admin: Option<bool>,
}

impl TryFrom<&Row> for UserVo {
	type Error = tokio_postgres::Error;
	fn try_from(value: &Row) -> Result<Self, Self::Error> {
		Ok(UserVo {
			id: value.try_get("id")?,
			username: value.try_get("username")?,
			phone_number: value.try_get("phone_number")?,
			email: value.try_get("email")?,
			password: value.try_get("password")?,
			gender: value.try_get("gender")?,
			avatar: value.try_get("avator")?,
			user_level: value.try_get("user_lv")?,
			introduction: value.try_get("introduction")?,
			subscribe: value.try_get("subscribe")?,
			create_time: value.try_get("create_time")?,
			update_time: value.try_get("update_time")?,
			update_user: value.try_get("update_user")?,
			is_boss: value.try_get("is_boss")?,
			access_token: value.try_get("access_token")?,
			code: value.try_get("code")?,
		})
	}
}
