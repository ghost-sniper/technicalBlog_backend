pub mod key_pair;
pub mod session;
pub mod sort;
pub mod user;
pub mod article;
pub mod tag;
pub mod category;

use chrono::Local;
use std::{
	fs::{self, OpenOptions},
	io::Write,
};
use thiserror::Error;

pub fn log_db_error(sql: &str, err: &dyn std::fmt::Display) {
	let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "logs".to_string());
	let _ = fs::create_dir_all(&log_dir);
	let log_path = format!("{}/db_error.log", log_dir.trim_end_matches('/'));
	let mut file = OpenOptions::new()
		.create(true)
		.append(true)
		.open(&log_path)
		.unwrap_or_else(|_| fs::File::create("db_error.log").unwrap());
	let line = format!(
		"{} [DB_ERROR] sql={} err={}",
		Local::now().format("%Y-%m-%d %H:%M:%S"),
		sql,
		err
	);
	let _ = writeln!(file, "{}", line);
	log::error!("DB_ERROR sql={} err={}", sql, err);
}

#[macro_export]
macro_rules! select_rows {
	($client:expr, $sql:expr, $params:expr, $model:ty) => {{
		match $client.query($sql, $params).await {
			Ok(rows) => rows
				.iter()
				.map(|row| <$model>::try_from(row).expect("Row 转 Model 失败"))
				.collect::<Vec<_>>(),
			Err(err) => {
				crate::dao::log_db_error($sql, &err);
				Vec::new()
			}
		}
	}};
}

#[macro_export]
macro_rules! select_row {
	($client:expr, $sql:expr, $params:expr, $model:ty) => {{
		match $client.query_one($sql, $params).await {
			Ok(row) => <$model>::try_from(&row).ok(),
			Err(err) => {
				crate::dao::log_db_error($sql, &err);
				None
			}
		}
	}};
}

#[derive(Debug, Error, Clone)]
pub enum DatabaseError {
	#[error("更新数据库失败")]
	UpdateFailed,
}
pub type DatabaseResult<T> = Result<T, DatabaseError>;
