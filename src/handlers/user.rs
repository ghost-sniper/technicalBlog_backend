use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use deadpool_postgres::Pool;

use crate::{
	models::{login_error::LoginError, user::UserLoginRequest},
	services::user::login,
};

pub async fn register_user() {
	todo!()
}

pub async fn login_user(
	pool: web::Data<Pool>,
	params: web::Json<UserLoginRequest>,
	request: HttpRequest,
) -> Result<HttpResponse, LoginError> {
	let username = required_field(&params.username, "未传入账号")?;
	let password = required_field(&params.password, "未提供密码")?;
	let public_key = required_field(&params.public_key, "未提供公钥")?;
	let aes_key = required_field(&params.aes_key, "未提供密钥")?;
	let iv = required_field(&params.iv, "未提供初始化向量")?;
	let client = pool.get().await.map_err(|error| {
		log::error!("获取数据库连接失败: {error}");
		LoginError::Internal
	})?;
	let user =
		login(&client, username.trim(), public_key, password, aes_key, iv).await?;
	Identity::login(&request.extensions(), user.id.to_string()).map_err(
		|error| {
			log::error!("设置Session失败: {error}");
			LoginError::Internal
		},
	)?;
	Ok(HttpResponse::Ok().json(user))
}

fn required_field<'a>(
	value: &'a Option<String>,
	message: &'static str,
) -> Result<&'a str, LoginError> {
	value
		.as_deref()
		.filter(|value| !value.trim().is_empty())
		.ok_or(LoginError::InvalidRequest(message))
}

pub fn login_json_config() -> web::JsonConfig {
	web::JsonConfig::default().error_handler(|_, _| {
		LoginError::InvalidRequest("登录请求格式错误，请检查请求参数").into()
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use actix_web::{App, http::header::CONTENT_TYPE, test};

	#[actix_web::test]
	async fn invalid_login_requests_use_the_error_contract() {
		let pool = crate::db::create_database_pool(
			"127.0.0.1".into(),
			"1".into(),
			"test".into(),
			"test".into(),
			"".into(),
		);
		let app = test::init_service(
			App::new()
				.app_data(web::Data::new(pool))
				.configure(crate::handlers::configure_app),
		)
		.await;
		let valid = serde_json::json!({"username": "test@example.com", "password": "encrypted", "public_key": "key", "aes_key": "key", "iv": "iv"});
		for (field, message) in [
			("username", "未传入账号"),
			("password", "未提供密码"),
			("public_key", "未提供公钥"),
			("aes_key", "未提供密钥"),
			("iv", "未提供初始化向量"),
		] {
			for value in [
				serde_json::Value::Null,
				serde_json::json!(""),
				serde_json::json!("   "),
			] {
				let mut payload = valid.clone();
				payload[field] = value;
				let response = test::call_service(
					&app,
					test::TestRequest::post()
						.uri("/api/login")
						.set_json(payload)
						.to_request(),
				)
				.await;
				assert_eq!(response.status().as_u16(), 400);
				let body: serde_json::Value = test::read_body_json(response).await;
				assert_eq!(body["code"], 40001);
				assert_eq!(body["message"], message);
			}
		}
		for payload in ["{", "[]", "{\"username\":123}"] {
			let response = test::call_service(
				&app,
				test::TestRequest::post()
					.uri("/api/login")
					.insert_header((CONTENT_TYPE, "application/json"))
					.set_payload(payload)
					.to_request(),
			)
			.await;
			assert_eq!(response.status().as_u16(), 400);
			let body: serde_json::Value = test::read_body_json(response).await;
			assert_eq!(body["code"], 40001);
			assert_eq!(body["message"], "登录请求格式错误，请检查请求参数");
		}
	}
}

pub async fn logout_user(identity: Identity) -> HttpResponse {
	identity.logout();
	HttpResponse::Ok().finish()
}

pub async fn update_user(identity: Option<Identity>) -> HttpResponse {
	if let Some(id) = identity {
		match id.id() {
			Ok(id) => HttpResponse::Ok().body(format!("Welcome, {}", id)),
			Err(e) => HttpResponse::Unauthorized().body(e.to_string()),
		}
	} else {
		HttpResponse::Unauthorized().finish()
	}
}

pub async fn get_code() {
	todo!()
}

pub async fn get_code_for_bind() {
	todo!()
}

pub async fn update_secret() {
	todo!()
}

pub async fn get_code_for_forget_password() {
	todo!()
}

pub async fn update_for_forget_password() {
	todo!()
}

pub async fn get_user_by_username() {
	todo!()
}

pub async fn subscribe() {
	todo!()
}
