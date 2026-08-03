use crate::{
	dao::{key_pair::get_private_key_by_public_key, user::query_user_by_email},
	models::user::User,
	utils::decrypt_e2ee,
};
use deadpool_postgres::Client;
pub async fn login(
	client: &Client,
	email: &str,
	public_key: &str,
	password: &str,
	aes_key: &str,
	iv: &str,
) -> anyhow::Result<User> {
	let user = query_user_by_email(client, email)
		.await
		.expect("获取用户信息失败");
	let private_key = get_private_key_by_public_key(client, public_key)
		.await
		.expect("获取私钥失败");
	let result = decrypt_e2ee(aes_key, password, iv, &private_key.as_str())
		.await
		.expect("解密失败");
	println!("解密后的密码：{}", result);
	println!("查询返回的密码：{}", user.password);
	if !user.password.eq(&result) {
		Err(anyhow::anyhow!("密码错误"))
	} else {
		Ok(user)
	}
}
