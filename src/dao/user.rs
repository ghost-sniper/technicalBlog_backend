use tokio_postgres::Client;

use crate::models::user::UserVo;
pub async fn query_user_by_account(conn: Client, account: &str) -> Option<UserVo> {
	todo!()
	// let sql = "SELECT * FROM user WHERE account = $1";
	// let mut stmt = conn.query(sql, &[account.to_owned()]).await.unwrap();
	// let rows = stmt.columns().unwrap();
	// for row in rows {
	// 	let user: UserVo = row.into();
	// 	return Some(user);
	// }
}
