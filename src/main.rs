use actix_cors::Cors;
use actix_web::{App, HttpServer, http, web};

use crate::db::create_database_pool;
pub mod db;
pub mod handlers;
pub mod models;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let db_pool = create_database_pool();
	HttpServer::new(move || {
		let cors = Cors::default()
			.allowed_origin("http://localhost:3000")
			.allowed_methods(vec!["GET", "POST", "HEAD", "PUT", "DELETE"])
			.allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
			.allowed_header(http::header::CONTENT_TYPE)
			.max_age(60);
		App::new()
			.wrap(cors)
			.configure(handlers::configure_app)
			.app_data(web::Data::new(db_pool.clone()))
	})
	.bind(("127.0.0.1", 8080))?
	.run()
	.await
}
