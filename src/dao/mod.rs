pub mod key_pair;
pub mod session;
pub mod sort;
pub mod user;
pub mod article;
pub mod tag;
pub mod category;

use thiserror::Error;
#[derive(Debug, Error, Clone)]
pub enum DatabaseError {
	#[error("更新数据库失败")]
	UpdateFailed,
}
pub type DatabaseResult<T> = Result<T, DatabaseError>;
