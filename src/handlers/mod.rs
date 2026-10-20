use actix_web::error::ErrorInternalServerError;
use actix_web::web;
use deadpool_postgres::Pool;

use crate::handlers::user::logout_user;

pub mod article;
pub mod category;
pub mod key_pair;
pub mod presentation;
pub mod sort;
pub mod tag;
pub mod user;

pub fn configure_app(cfg: &mut web::ServiceConfig) {
	let routes = web::scope("/api")
		.route("/categories", web::get().to(category::get_all_categories))
		.route("/tags", web::get().to(tag::get_all_tags))
		.route("/tag/add", web::post().to(tag::insert_tag))
		.route("/webInfo/listLabel", web::get().to(sort::list_label))
		.route("/publicKey", web::get().to(key_pair::get_public_key))
		.service(
			web::resource("/login")
				.app_data(user::login_json_config())
				.route(web::post().to(user::login_user)),
		)
		.route("/logout", web::post().to(logout_user))
		.route("/user/me", web::get().to(user::update_user))
		.route("/article/{id}", web::get().to(article::get_article))
		.route(
			"/list/article",
			web::get().to(article::get_all_short_article),
		);
	cfg.service(routes);
}

async fn get_db_client(
	pool: web::Data<Pool>,
) -> Result<deadpool_postgres::Client, actix_web::Error> {
	pool.get().await.map_err(|e| {
		eprintln!("Failed to get database connection: {}", e);
		ErrorInternalServerError("Database connection error")
	})
}
