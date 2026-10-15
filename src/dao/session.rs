use chrono::{DateTime, Utc};
use deadpool_postgres::Client;

use crate::dao::log_db_error;

pub async fn query_session_by_user_id(
	conn: &Client,
	user_id: i32,
) -> Option<String> {
	let sql = "SELECT token FROM user_session WHERE user_id = $1";
	match conn.query_one(sql, &[&user_id]).await {
		Ok(row) => row.try_get("token").ok(),
		Err(err) => {
			log_db_error(sql, &err);
			None
		},
	}
}

pub async fn quere_expires_time_by_token(
	conn: &Client,
	token: &str,
) -> Option<DateTime<Utc>> {
	let sql = "SELECT expires_at FROM user_session WHERE token = $1";
	match conn.query_one(sql, &[&token]).await {
		Ok(row) => row.try_get("expires_at").ok(),
		Err(err) => {
			log_db_error(sql, &err);
			None
		},
	}
}

pub async fn update_expires_time(
	conn: &Client,
	expires_time: DateTime<Utc>,
	token: &str,
) -> Result<u64, tokio_postgres::Error> {
	conn.execute(
		"UPDATE poetize.user_session SET expires_at = $1 WHERE token = $2",
		&[&expires_time, &token],
	)
	.await
}
