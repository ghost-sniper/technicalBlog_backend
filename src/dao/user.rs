use tokio_postgres::Client;

use crate::{models::user::User, select_row};
pub async fn query_user_by_email(conn: &Client, email: &str) -> Option<User> {
	select_row!(
		conn,
		"SELECT id, username, password,phone_number,email, user_status,gender,open_id,avatar,admire,subscribe,introduction,user_type, user_lv AS user_level, create_time, update_time,update_by, deleted FROM poetize.user WHERE email = $1",
		&[&email],
		User
	)
}
