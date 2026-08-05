use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;

pub fn create_database_pool(host: String,port:String,dbname:String,user:String,password:String) -> Pool {
	let mut db_cfg = Config::new();
	db_cfg.host = Some(host);
	db_cfg.port = Some(port.parse::<u16>().unwrap());
	db_cfg.dbname = Some(dbname);
	db_cfg.user = Some(user);
	db_cfg.password = Some(password);
	db_cfg
		.create_pool(Some(Runtime::Tokio1), NoTls)
		.expect("Failed to create database pool")
}
