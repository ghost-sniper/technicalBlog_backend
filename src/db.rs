use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;

pub fn create_database_pool() -> Pool {
	let mut db_cfg = Config::new();
	db_cfg.host = Some("localhost".to_string());
	db_cfg.dbname = Some("mydb".to_string());
	db_cfg.user = Some("postgres".to_string());
	db_cfg.password = Some("secret".to_string());
	db_cfg
		.create_pool(Some(Runtime::Tokio1), NoTls)
		.expect("Failed to create database pool")
}
