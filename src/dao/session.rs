use chrono::{DateTime, Utc};
use deadpool_postgres::Client;

pub async fn query_session_by_user_id(
	conn: &Client,
	user_id: i32,
) -> Option<String> {
	let result = conn
		.query_one(
			"SELECT token FROM user_session WHERE user_id = $1",
			&[&user_id],
		)
		.await;
	result.unwrap().try_get("token").unwrap()
}

pub async fn quere_expires_time_by_token(
	conn: &Client,
	token: &str,
) -> Option<DateTime<Utc>> {
	conn.query_one(
		"SELECT expires_at FROM user_session WHERE token = $1",
		&[&token],
	)
	.await
	.unwrap()
	.try_get("expires_at")
	.unwrap()
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
