use crate::{
	dao::{key_pair::get_private_key_by_public_key, user::query_user_by_email},
	models::{login_error::LoginError, user::User},
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
) -> Result<User, LoginError> {
	let user = query_user_by_email(client, email).await.map_err(|error| {
		log::error!("查询登录用户失败: {error}");
		LoginError::Internal
	})?;
	let user = require_active_user(user)?;
	let private_key = get_private_key_by_public_key(client, public_key)
		.await
		.map_err(|error| {
			log::error!("查询登录密钥失败: {error}");
			LoginError::Internal
		})?
		.ok_or(LoginError::InvalidLoginKey)?;
	verify_password(user, &private_key, password, aes_key, iv).await
}

fn require_active_user(user: Option<User>) -> Result<User, LoginError> {
	let user = user
		.filter(|user| user.deleted == 0)
		.ok_or(LoginError::UserNotFound)?;
	// Database convention: 1 = active, 0 = frozen. Unknown states cannot log in.
	if user.user_status != 1 {
		return Err(LoginError::AccountFrozen);
	}
	Ok(user)
}

async fn verify_password(
	user: User,
	private_key: &str,
	password: &str,
	aes_key: &str,
	iv: &str,
) -> Result<User, LoginError> {
	let plaintext = decrypt_e2ee(aes_key, password, iv, private_key)
		.await
		.map_err(|_| LoginError::InvalidEncryptedPayload)?;
	if user.password != plaintext {
		return Err(LoginError::WrongPassword);
	}
	Ok(user)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn user(status: i16, deleted: i16) -> User {
		serde_json::from_value(serde_json::json!({
			"id": 1, "username": "test-user", "password": "test-password-中文-🔐",
			"user_status": status, "gender": 0, "user_type": 0, "user_level": 0,
			"create_time": "2026-01-01T00:00:00", "update_time": "2026-01-01T00:00:00",
			"deleted": deleted
		}))
		.unwrap()
	}

	#[test]
	fn missing_deleted_and_frozen_users_cannot_log_in() {
		assert_eq!(
			require_active_user(None).unwrap_err(),
			LoginError::UserNotFound
		);
		assert_eq!(
			require_active_user(Some(user(1, 1))).unwrap_err(),
			LoginError::UserNotFound
		);
		assert_eq!(
			require_active_user(Some(user(0, 0))).unwrap_err(),
			LoginError::AccountFrozen
		);
		assert_eq!(
			require_active_user(Some(user(2, 0))).unwrap_err(),
			LoginError::AccountFrozen
		);
		assert!(require_active_user(Some(user(1, 0))).is_ok());
	}

	#[actix_web::test]
	async fn password_errors_are_distinct_from_encryption_errors() {
		let fixtures: serde_json::Value =
			serde_json::from_str(include_str!("../../tests/fixtures/e2ee.json"))
				.unwrap();
		let fixture = &fixtures[0];
		let key = fixture["private_key"].as_str().unwrap();
		let password = fixture["cipher_text"].as_str().unwrap();
		let aes_key = fixture["aes_key"].as_str().unwrap();
		let iv = fixture["iv"].as_str().unwrap();
		assert!(
			verify_password(user(1, 0), key, password, aes_key, iv)
				.await
				.is_ok()
		);
		let mut wrong_password = user(1, 0);
		wrong_password.password = "another password".to_owned();
		assert_eq!(
			verify_password(wrong_password, key, password, aes_key, iv)
				.await
				.unwrap_err(),
			LoginError::WrongPassword
		);
		assert_eq!(
			verify_password(user(1, 0), key, password, "invalid base64!", iv)
				.await
				.unwrap_err(),
			LoginError::InvalidEncryptedPayload
		);
	}
}
