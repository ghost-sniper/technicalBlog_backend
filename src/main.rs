use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::{App, HttpServer, http, web};
use actix_web::cookie::Key;
use crate::db::create_database_pool;
pub mod dao;
pub mod db;
pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let db_pool = create_database_pool();
	let secret_key = Key::generate();
	HttpServer::new(move || {
		let store = CookieSessionStore::default();
		let cors = Cors::default()
			.allowed_origin("http://localhost:3000")
			.allowed_methods(vec!["GET", "POST", "HEAD", "PUT", "DELETE"])
			.allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
			.allowed_header(http::header::CONTENT_TYPE)
			.max_age(60);
		App::new()
			.wrap(cors).wrap(IdentityMiddleware::default()).wrap(SessionMiddleware::new(store, secret_key.clone()))
			.configure(handlers::configure_app)
			.app_data(web::Data::new(db_pool.clone()))
	})
	.bind(("127.0.0.1", 8600))?
	.run()
	.await
}
