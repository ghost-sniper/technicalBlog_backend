use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
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
	let Some(username) = params.username.clone() else {
		return HttpResponse::BadRequest().body("未传入账号");
	};
	let Some(password) = params.password.clone() else {
		return HttpResponse::BadRequest().body("未提供密码");
	};
	let Some(public_key) = params.public_key.clone() else {
		return HttpResponse::BadRequest().body("未提供公钥");
	};
	let Some(aes_key) = params.aes_key.clone() else {
		return HttpResponse::BadRequest().body("未提供密钥");
	};
	let Some(iv) = params.iv.clone() else {
		return HttpResponse::BadRequest().body("未提供初始化向量");
	};
	let client = match pool.get().await {
		Ok(client) => client,
		Err(e) => {
			log::error!("获取数据库连接失败: {e}");
			return HttpResponse::InternalServerError().finish();
		}
	};
	let user = match login(&client, &username, &public_key, &password, &aes_key, &iv).await {
		Ok(user) => user,
		Err(e) => {
			log::warn!("登录失败: {e}");
			return HttpResponse::Unauthorized().body("账号或密码错误");
		}
	};
	match Identity::login(&request.extensions(), user.id.to_string()) {
		Ok(_) => HttpResponse::Ok().json(user),
		Err(e) => {
			log::error!("设置Session失败: {e}");
			HttpResponse::InternalServerError().finish()
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
