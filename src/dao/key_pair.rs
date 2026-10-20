use deadpool_postgres::Client;

use crate::{dao::log_db_error, models::key_pair::RsaKeyPair};

pub async fn get_public_key(conn: &Client) -> Option<String> {
	let sql = "SELECT public_key FROM poetize.key_pair ORDER BY RANDOM() LIMIT 1";
	match conn.query_one(sql, &[]).await {
		Ok(row) => row.try_get("public_key").ok(),
		Err(err) => {
			log_db_error(sql, &err);
			None
		}
	}
}

pub async fn get_private_key_by_public_key(
	conn: &Client,
	public_key: &str,
) -> Result<Option<String>, tokio_postgres::Error> {
	let sql = "SELECT private_key FROM poetize.key_pair WHERE public_key = $1";
	let row = conn.query_opt(sql, &[&public_key]).await?;
	row.as_ref()
		.map(|row| row.try_get("private_key"))
		.transpose()
}

pub async fn get_key_pair(conn: &Client) -> Option<Vec<RsaKeyPair>> {
	let sql = "SELECT public_key, private_key FROM poetize.key_pair";
	match conn.query(sql, &[]).await {
		Ok(rows) => Some(
			rows.iter()
				.filter_map(|i| RsaKeyPair::try_from(i).ok())
				.collect(),
		),
		Err(err) => {
			log_db_error(sql, &err);
			None
		}
	}
}
