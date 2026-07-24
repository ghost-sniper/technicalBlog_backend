use actix_web::{HttpResponse, web};

use crate::models::user::UserLoginParams;

pub async fn register_user() {
	todo!()
}

pub async fn login_user(params: web::Query<UserLoginParams>) -> HttpResponse {
	if let None = params.account {
		return HttpResponse::BadRequest().body("未传入账号");
	};
	if let None = params.password {
		return HttpResponse::BadRequest().body("未提供密码");
	};
	HttpResponse::Ok().finish()
}

pub async fn token() {
	todo!()
}

pub async fn logout_user() {
	todo!()
}

pub async fn update_user() {
	todo!()
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
