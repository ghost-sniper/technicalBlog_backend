use crate::{dao::log_db_error, models::sort::Sort};
use deadpool_postgres::Client;

pub async fn query_all_label(conn: &Client) -> Option<Vec<Sort>> {
	let sql = "select s.id , s.sort_name , s.sort_description , s.sort_type , s.priority from poetize.sort s";
	match conn.query(sql, &[]).await {
		Ok(query_result) => Some(
			query_result
				.iter()
				.map(|r| {
					println!("{:?}", r);
					Sort::try_from(r).ok()
				})
				.collect::<Option<Vec<_>>>()?,
		),
		Err(err) => {
			log_db_error(sql, &err);
			None
		},
	}
}
