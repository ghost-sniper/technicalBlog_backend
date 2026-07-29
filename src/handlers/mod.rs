use actix_web::web;
use thiserror::Error;

pub mod category;
pub mod key_pair;
pub mod sort;
pub mod tag;
pub mod user;
#[derive(Error, Debug, Clone)]
pub enum LoginError {
	#[error("未传入账号")]
	AccountNotProvided,
	#[error("未提供密码")]
	PasswordNotProvided,
}

pub type LoginResult<T> = Result<T, LoginError>;
pub fn configure_app(cfg: &mut web::ServiceConfig) {
	let public_routes = web::scope("")
		.route("/categories", web::get().to(category::get_all_categories))
		.route("/tags", web::get().to(tag::get_all_tags))
		.route("/tag/add", web::post().to(tag::insert_tag))
		.route("/webInfo/listLabel", web::get().to(sort::list_label))
		.route("/publicKey", web::get().to(key_pair::get_public_key))
		.route("/login", web::post().to(user::login_user));
	cfg.service(public_routes);
}
