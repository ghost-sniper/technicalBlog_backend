use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
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
impl UserVo {
	pub fn new(
		id: Option<i64>,
		username: String,
		phone_number: Option<String>,
		email: Option<String>,
		password: String,
		gender: Option<i16>,
		avatar: Option<String>,
		user_level: Option<i16>,
		introduction: Option<String>,
		subscribe: Option<String>,
		create_time: Option<DateTime<FixedOffset>>,
		update_time: Option<DateTime<FixedOffset>>,
		update_user: Option<String>,
		is_boss: Option<bool>,
		access_token: Option<String>,
		code: Option<String>,
	) -> Self {
		Self {
			id,
			username,
			phone_number,
			email,
			password,
			gender,
			avatar,
			user_level,
			introduction,
			subscribe,
			create_time,
			update_time,
			update_user,
			is_boss,
			access_token,
			code,
		}
	}
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserLoginParams {
	pub account: Option<String>,
	pub password: Option<String>,
	pub is_admin: Option<bool>,
}
