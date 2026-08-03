use crate::db::create_database_pool;
use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;
use actix_web::{App, HttpServer, http, web};
use chrono::Local;
use env_logger::Env;
use log::info;
use std::io::Write;
pub mod dao;
pub mod db;
pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	init_logger();
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
			.wrap(cors)
			.wrap(IdentityMiddleware::default())
			.wrap(SessionMiddleware::new(store, secret_key.clone()))
			.configure(handlers::configure_app)
			.app_data(web::Data::new(db_pool.clone()))
	})
	.bind(("127.0.0.1", 8600))?
	.run()
	.await
}
fn init_logger() {
	let env = Env::default().filter_or("MY_LOG_LEVEL", "debug");
	env_logger::Builder::from_env(env)
		.format(|buf, record| {
			let level_color = match record.level() {
				// log::Level::Error => {
				// 	Color::Rgb(RgbColor::try_from((255, 0, 0)).unwrap())
				// }
				// log::Level::Warn => {
				// 	Color::Rgb(RgbColor::try_from((255, 255, 0)).unwrap())
				// }
				// log::Level::Info => {
				// 	Color::Rgb(RgbColor::try_from((0, 255, 0)).unwrap())
				// }
				// log::Level::Debug | log::Level::Trace => {
				// 	Color::Rgb(RgbColor::try_from((0, 255, 255)).unwrap())
				// }
				log::Level::Error => "\x1b[31mERROR\x1b[0m", // 红色
				log::Level::Warn => "\x1b[33mWARN \x1b[0m", // 黄色
				log::Level::Info => "\x1b[32mINFO \x1b[0m", // 绿色
				log::Level::Debug => "\x1b[34mDEBUG\x1b[0m", // 蓝色
				log::Level::Trace => "\x1b[90mTRACE\x1b[0m", // 灰色
			};
			// let level_style =
			// 	Style::new().fg_color(Some(level_color).into()).bold();
			writeln!(
				buf,
				"{} {} [{}] {}",
				Local::now().format("%Y-%m-%d %H:%M:%S"),
				level_color,
				record.module_path().unwrap_or("<unnamed>"),
				record.args()
			)
		})
		.filter(None, log::LevelFilter::Debug)
		.init();
	info!("env_logger set up");
}
