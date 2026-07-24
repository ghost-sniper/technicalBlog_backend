use actix_web::web;
use thiserror::Error;
pub mod category;
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
	cfg.route("/categories", web::get().to(category::get_all_categories))
		.route("/tags", web::get().to(tag::get_all_tags))
		.route("/tag/add", web::post().to(tag::insert_tag));
}
