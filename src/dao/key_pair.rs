use deadpool_postgres::Client;

use crate::models::key_pair::RsaKeyPair;

pub async fn get_public_key(conn: &Client) -> Option<String> {
	let result = conn
		.query_one(
			"SELECT public_key FROM poetize.key_pair ORDER BY RANDOM() LIMIT 1",
			&[],
		)
		.await;
	result.map(|i| i.get("public_key")).ok()
}

pub async fn get_private_key_by_public_key(
	conn: &Client,
	public_key: &str,
) -> Option<String> {
	let result = conn
		.query_one(
			"SELECT private_key FROM poetize.key_pair WHERE public_key = $1",
			&[&public_key],
		)
		.await;
	result.map(|i| i.get("private_key")).ok()
}

pub async fn get_key_pair(conn: &Client) -> Option<Vec<RsaKeyPair>> {
	let result = conn
		.query("SELECT public_key, private_key FROM poetize.key_pair", &[])
		.await
		.unwrap();

	result
		.iter()
		.map(|i| RsaKeyPair::try_from(i).ok())
		.collect()
}
