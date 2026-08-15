pub mod key_pair;
pub mod session;
pub mod sort;
pub mod user;
pub mod article;
pub mod tag;
pub mod category;

#[macro_export]
macro_rules! select_rows {
	($client:expr, $sql:expr, $params:expr, $model:ty) => {{
		let rows = $client.query($sql, $params).await.expect("数据库查询失败");
		rows.iter()
			.map(|row| <$model>::try_from(row).expect("Row 转 Model 失败"))
			.collect::<Vec<_>>()
	}};
}

#[macro_export]
macro_rules! select_row {
	($client:expr, $sql:expr, $params:expr, $model:ty) => {{
		match $client.query_one($sql, $params).await {
			Ok(row) => <$model>::try_from(&row).ok(),
			Err(_) => None,
		}
	}};
}

use thiserror::Error;
#[derive(Debug, Error, Clone)]
pub enum DatabaseError {
	#[error("更新数据库失败")]
	UpdateFailed,
}
pub type DatabaseResult<T> = Result<T, DatabaseError>;
