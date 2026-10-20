use tokio_postgres::Client;

use crate::models::user::User;
pub async fn query_user_by_email(
	conn: &Client,
	email: &str,
) -> Result<Option<User>, tokio_postgres::Error> {
	let row = conn.query_opt(
		"SELECT id, username, password,phone_number,email, user_status,gender,open_id,avatar,admire,subscribe,introduction,user_type, user_lv AS user_level, create_time, update_time,update_by, deleted FROM poetize.user WHERE email = $1 AND deleted = 0",
		&[&email],
	).await?;
	row.as_ref().map(User::try_from).transpose()
}
