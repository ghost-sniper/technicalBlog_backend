use tokio_postgres::Client;

use crate::models::user::UserVo;
pub async fn query_user_by_account(
	conn: Client,
	account: &str,
) -> Option<Vec<UserVo>> {
	let rows = conn
		.query("SELECT * FROM user WHERE account = $1", &[&account])
		.await
		.unwrap();
	if rows.is_empty() {
		return None;
	} else {
		return rows.iter().map(|i| UserVo::try_from(i).ok()).collect();
	}
}
