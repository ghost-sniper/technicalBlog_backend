use actix_web::{App, HttpServer, web};

use crate::db::create_database_pool;
pub mod db;
pub mod handlers;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let db_pool = create_database_pool();
	HttpServer::new(move || {
		App::new()
			.configure(handlers::configure_app)
			.app_data(web::Data::new(db_pool.clone()))
	})
	.bind(("127.0.0.1", 8080))?
	.run()
	.await
}
