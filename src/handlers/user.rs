use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use deadpool_postgres::Pool;
use tokio_postgres::Row;

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

#[derive(serde::Serialize, blog::TryFromRow)]
pub struct ProfileUser {
	#[serde(skip_serializing)]
	pub user_type: i16,
	pub id: i32,
	pub username: String,
	pub email: Option<String>,
	pub phone_number: Option<String>,
	pub avatar: Option<String>,
	pub introduction: Option<String>,
	pub user_level: i16,
	pub create_time: chrono::NaiveDateTime,
	#[serde(skip_serializing)]
	pub user_status: i16,
	#[serde(skip_serializing)]
	pub deleted: i16,
}

#[derive(serde::Serialize, blog::TryFromRow)]
pub struct AuthorArticle {
	pub id: i32,
	pub article_title: String,
	pub prospectus: Option<String>,
	pub view_count: i32,
	pub like_count: i32,
	pub view_type: String,
	pub create_time: Option<chrono::NaiveDateTime>,
	pub update_time: Option<chrono::NaiveDateTime>,
}

#[derive(serde::Serialize)]
pub struct ProfileResponse {
	pub can_manage_tags: bool,
	pub user: ProfileUser,
	pub articles: Vec<AuthorArticle>,
	pub count: usize,
}

pub async fn update_user(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
) -> HttpResponse {
	let Some(identity) = identity else {
		return HttpResponse::Unauthorized()
			.json(serde_json::json!({"code":40100,"message":"请先登录"}));
	};
	let id = match identity
		.id()
		.ok()
		.and_then(|value| value.parse::<i32>().ok())
	{
		Some(id) => id,
		None => {
			identity.logout();
			return HttpResponse::Unauthorized().json(
				serde_json::json!({"code":40100,"message":"登录已失效，请重新登录"}),
			);
		}
	};
	let client = match pool.get().await {
		Ok(client) => client,
		Err(error) => {
			log::error!("个人主页连接数据库失败: {error}");
			return profile_service_error();
		}
	};
	let user = match client.query_opt("SELECT id,username,email,phone_number,avatar,introduction,user_lv AS user_level,create_time,user_status,deleted,user_type FROM public.user WHERE id=$1", &[&id]).await {
        Ok(Some(row)) => match ProfileUser::try_from(&row) {
            Ok(user) => user,
            Err(error) => {log::error!("个人资料解析失败: {error}"); return profile_service_error();}
        },
        Ok(None) => { identity.logout(); return HttpResponse::Unauthorized().json(serde_json::json!({"code":40100,"message":"登录已失效，请重新登录"})); },
        Err(error) => {log::error!("个人资料查询失败: {error}"); return profile_service_error();}
    };
	if user.deleted != 0 || user.user_status != 1 {
		identity.logout();
		return HttpResponse::Forbidden().json(
			serde_json::json!({"code":40301,"message":"账号已被冻结或删除，请联系管理员"}),
		);
	}
	// Author identity comes exclusively from the authenticated session.
	let articles = match client.query("SELECT id,article_title,prospectus,view_count,like_count,view_type,create_time,update_time FROM public.article WHERE user_id=$1 AND deleted=0 ORDER BY update_time DESC NULLS LAST, id DESC", &[&id]).await {
        Ok(rows) => match rows.iter().map(AuthorArticle::try_from).collect::<Result<Vec<_>,_>>() {
            Ok(articles) => articles,
            Err(error) => {log::error!("作者文章解析失败: {error}"); return profile_service_error();}
        },
        Err(error) => {log::error!("作者文章查询失败: {error}"); return profile_service_error();}
    };
	HttpResponse::Ok().json(ProfileResponse {
		can_manage_tags: super::tag_admin::admin_type() == user.user_type,
		count: articles.len(),
		user,
		articles,
	})
}

fn profile_service_error() -> HttpResponse {
	HttpResponse::InternalServerError().json(
		serde_json::json!({"code":50001,"message":"个人主页暂时不可用，请稍后重试"}),
	)
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

#[cfg(test)]
mod profile_tests {
	use super::*;
	use actix_identity::IdentityMiddleware;
	use actix_session::{SessionMiddleware, storage::RedisSessionStore};
	use actix_web::cookie::Key;
	use actix_web::{App, test};

	#[actix_web::test]
	async fn anonymous_profile_request_is_rejected() {
		let pool = crate::db::create_database_pool(
			"127.0.0.1".into(),
			"1".into(),
			"unused".into(),
			"unused".into(),
			"".into(),
		);
		let response = update_user(None, web::Data::new(pool)).await;
		assert_eq!(response.status().as_u16(), 401);
		let bytes = actix_web::body::to_bytes(response.into_body())
			.await
			.unwrap();
		let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
		assert_eq!(body["code"], 40100);
	}

	// This route exists only in the isolated test app, never in the real server.
	async fn test_session(request: HttpRequest, id: web::Data<i32>) -> HttpResponse {
		Identity::login(&request.extensions(), id.to_string()).unwrap();
		HttpResponse::Ok().finish()
	}

	#[actix_web::test]
	#[ignore = "requires configured PostgreSQL and Redis; reads only database data"]
	async fn live_session_profile_is_private_and_author_scoped() {
		dotenvy::dotenv().ok();
		let pool = crate::db::create_database_pool(
			std::env::var("DATABASE_HOST").unwrap(),
			std::env::var("DATABASE_PORT").unwrap(),
			std::env::var("DATABASE_NAME").unwrap(),
			std::env::var("DATABASE_USER").unwrap(),
			std::env::var("DATABASE_USER_PASSWORD").unwrap(),
		);
		let client = pool.get().await.unwrap();
		let id:i32 = client.query_one("SELECT id FROM public.user WHERE user_status=1 AND deleted=0 ORDER BY id LIMIT 1",&[]).await.unwrap().get(0);
		let expected: Vec<i32> = client.query("SELECT id FROM public.article WHERE user_id=$1 AND deleted=0 ORDER BY update_time DESC NULLS LAST,id DESC",&[&id]).await.unwrap().iter().map(|row|row.get(0)).collect();
		let store = RedisSessionStore::new(
			std::env::var("REDIS_URL")
				.unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
		)
		.await
		.unwrap();
		let app = test::init_service(
			App::new()
				.app_data(web::Data::new(pool))
				.app_data(web::Data::new(id))
				.wrap(IdentityMiddleware::default())
				.wrap(
					SessionMiddleware::builder(store, Key::generate())
						.cookie_secure(false)
						.build(),
				)
				.route("/_test/session", web::post().to(test_session))
				.route("/api/user/me", web::get().to(update_user))
				.route("/api/logout", web::post().to(logout_user)),
		)
		.await;
		let response = test::call_service(
			&app,
			test::TestRequest::post().uri("/_test/session").to_request(),
		)
		.await;
		let cookie = response.response().cookies().next().unwrap().into_owned();
		let response = test::call_service(
			&app,
			test::TestRequest::get()
				.uri("/api/user/me?user_id=-999")
				.cookie(cookie.clone())
				.to_request(),
		)
		.await;
		assert_eq!(response.status().as_u16(), 200);
		let body: serde_json::Value = test::read_body_json(response).await;
		assert_eq!(body["user"]["id"], id);
		for sensitive in ["password", "open_id", "user_status", "deleted"] {
			assert!(body["user"].get(sensitive).is_none());
		}
		let actual: Vec<i32> = body["articles"]
			.as_array()
			.unwrap()
			.iter()
			.map(|row| row["id"].as_i64().unwrap() as i32)
			.collect();
		assert_eq!(actual, expected);
		assert_eq!(body["count"].as_u64().unwrap() as usize, actual.len());
		let response = test::call_service(
			&app,
			test::TestRequest::post()
				.uri("/api/logout")
				.cookie(cookie.clone())
				.to_request(),
		)
		.await;
		assert_eq!(response.status().as_u16(), 200);
		let response = test::call_service(
			&app,
			test::TestRequest::get()
				.uri("/api/user/me")
				.cookie(cookie)
				.to_request(),
		)
		.await;
		assert_eq!(response.status().as_u16(), 401);
	}
}
