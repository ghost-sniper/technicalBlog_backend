use std::fmt::format;
use std::mem::forget;
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, HttpMessage};
use deadpool_postgres::Pool;

use crate::{models::user::UserLoginRequest, services::user::login};

pub async fn register_user() {
	todo!()
}

pub async fn login_user(
	pool: web::Data<Pool>,
	params: web::Json<UserLoginRequest>,
	request: HttpRequest,
) -> HttpResponse {
	println!("开始登陆操作");
	if let None = params.username {
		return HttpResponse::BadRequest().body("未传入账号");
	};
	if let None = params.password {
		return HttpResponse::BadRequest().body("未提供密码");
	};
	let client = pool.get().await.unwrap();
	let params = params.into_inner();
	println!("{:?}", params.username.as_ref());
	let user = login(
		&client,
		params.username.expect("").as_str(),
		params.public_key.expect("").as_str(),
		params.password.expect("").as_str(),
		params.aes_key.expect("").as_str(),
		params.iv.expect("").as_str(),
	)
	.await
	.unwrap();
	Identity::login(&request.extensions(), user.id.to_string()).expect("设置Session失败");
	HttpResponse::Ok().json(user)
}


pub async fn logout_user(identity: Identity) -> HttpResponse {
	identity.logout();
	HttpResponse::Ok().finish()
}

pub async fn update_user(identity: Option<Identity>) -> HttpResponse {
	if let Some(id) = identity {
		match id.id() {
			Ok(id) => {HttpResponse::Ok().body(format!("Welcome, {}",id))}
			Err(e)=> HttpResponse::Unauthorized().body(e.to_string())
		}
	}else {
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
