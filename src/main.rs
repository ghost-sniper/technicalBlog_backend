use actix_web::{App, HttpServer};
pub mod handlers;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	HttpServer::new(|| App::new().configure(handlers::configure_app))
		.bind(("127.0.0.1", 8080))?
		.run()
		.await
}
