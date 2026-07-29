use tokio_postgres::Client;

use crate::models::user::User;
pub async fn query_user_by_email(conn: &Client, email: &str) -> Option<User> {
	let row = conn
		.query_one("SELECT id, username, password,phone_number,email, user_status,gender,open_id,avatar,admire,subscribe,introduction,user_type, user_lv, create_time, update_time,update_by, deleted FROM poetize.user WHERE email = $1", &[&email]).await.expect("查询客户信息失败");
	let user = User::try_from(&row).ok();
	user
}
