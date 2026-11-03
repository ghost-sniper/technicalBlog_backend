use actix_web::error::ErrorInternalServerError;
use actix_web::web;
use deadpool_postgres::Pool;

use crate::handlers::user::logout_user;

pub mod article;
pub mod article_edit;
mod article_image;
pub mod category;
pub mod key_pair;
pub mod presentation;
pub mod sort;
pub mod tag;
pub mod tag_admin;
pub mod user;

pub fn configure_app(cfg: &mut web::ServiceConfig) {
	let routes = web::scope("/api")
		.route("/categories", web::get().to(category::get_all_categories))
		.route("/tags", web::get().to(tag::get_all_tags))
		.service(
			web::resource("/admin/tags")
				.route(web::get().to(tag_admin::get))
				.route(web::post().to(tag_admin::create)),
		)
		.service(
			web::resource("/admin/tags/{id}")
				.route(web::put().to(tag_admin::rename))
				.route(web::delete().to(tag_admin::delete)),
		)
		.route("/tag/add", web::post().to(tag_admin::create))
		.route("/webInfo/listLabel", web::get().to(sort::list_label))
		.route("/publicKey", web::get().to(key_pair::get_public_key))
		.service(
			web::resource("/login")
				.app_data(user::login_json_config())
				.route(web::post().to(user::login_user)),
		)
		.route("/logout", web::post().to(logout_user))
		.route("/user/me", web::get().to(user::update_user))
		.service(
			web::resource("/article/{id}/edit")
				.app_data(web::JsonConfig::default().limit(2_000_000))
				.route(web::get().to(article_edit::get_editable))
				.route(web::put().to(article_edit::save)),
		)
		.service(
			web::resource("/article/{id}/images")
				.app_data(web::PayloadConfig::new(article_image::MAX_IMAGE_BYTES))
				.route(web::post().to(article_image::upload)),
		)
		.route("/article-images/{id}", web::get().to(article_image::read))
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
