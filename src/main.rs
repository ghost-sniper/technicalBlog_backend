use crate::db::create_database_pool;
use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::{Key, SameSite};
use actix_web::{App, HttpServer, http, web};
use base64::{Engine as _, engine::general_purpose};
use chrono::Local;
use env_logger::Env;
use log::info;
use std::io::Write;
use dotenvy::dotenv;
pub mod dao;
pub mod db;
pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	init_logger();
	dotenv().ok();
	let database_host = std::env::var("DATABASE_HOST").unwrap_or("127.0.0.1".to_string());
	let database_port = std::env::var("DATABASE_PORT").unwrap_or("5432".to_string());
	let database_user = std::env::var("DATABASE_USER").unwrap_or("zion_app".to_string());
	let database_name = std::env::var("DATABASE_NAME").unwrap_or("zion".to_string());
	let database_user_passwd = std::env::var("DATABASE_USER_PASSWORD").unwrap_or("".to_string());
	let db_pool = create_database_pool(database_host,database_port,database_name,database_user,database_user_passwd);
	let redis_url = std::env::var("REDIS_URL")
		.unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
	let redis_store = RedisSessionStore::new(redis_url)
		.await
		.expect("Failed to connect Redis session store, check REDIS_URL");
	let secret_b64 = std::env::var("SESSION_SECRET").expect("Missing SESSION_SECRET env var");
	let key_material = general_purpose::STANDARD
		.decode(secret_b64.trim())
		.expect("SESSION_SECRET is not valid base64");
	assert!(
		key_material.len() >= 64,
		"SESSION_SECRET must decode to >= 64 bytes, generate one with `openssl rand -base64 64`"
	);
	let secret_key = Key::from(&key_material);
	let cookie_secure = std::env::var("COOKIE_SECURE")
		.map(|v| v == "true")
		.unwrap_or(false);
	HttpServer::new(move || {
		let cors = Cors::default()
			.allowed_origin("http://localhost:3000")
			.allowed_methods(vec!["GET", "POST", "HEAD", "PUT", "DELETE"])
			.allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
			.allowed_header(http::header::CONTENT_TYPE)
			.max_age(60);
		App::new()
			.wrap(cors)
			.wrap(IdentityMiddleware::default())
			.wrap(
				SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
					.cookie_http_only(true)
					.cookie_same_site(SameSite::Lax)
					.cookie_secure(cookie_secure)
					.build(),
			)
			.configure(handlers::configure_app)
			.app_data(web::Data::new(db_pool.clone()))
	})
	.bind(("0.0.0.0", 8080))?
	.run()
	.await
}
fn init_logger() {
	let env = Env::default().filter_or("MY_LOG_LEVEL", "debug");
	env_logger::Builder::from_env(env)
		.format(|buf, record| {
			let level_color = match record.level() {
				log::Level::Error => "\x1b[31mERROR\x1b[0m", // 红色
				log::Level::Warn => "\x1b[33mWARN \x1b[0m", // 黄色
				log::Level::Info => "\x1b[32mINFO \x1b[0m", // 绿色
				log::Level::Debug => "\x1b[34mDEBUG\x1b[0m", // 蓝色
				log::Level::Trace => "\x1b[90mTRACE\x1b[0m", // 灰色
			};
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
