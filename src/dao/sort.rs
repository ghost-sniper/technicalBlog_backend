use crate::models::sort::Sort;
use deadpool_postgres::Client;
pub async fn query_all_label(conn: &Client) -> Option<Vec<Sort>> {
	let query_result = conn.query("select s.id , s.sort_name , s.sort_description , s.sort_type , s.priority from poetize.sort s", &[]).await.expect("Query DataBase Failed. Please Check Database configuration");
	query_result
		.iter()
		.map(|r| {
			println!("{:?}", r);
			Sort::try_from(r).ok()
		})
		.collect()
}
