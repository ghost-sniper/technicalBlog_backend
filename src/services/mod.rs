pub mod sort;
pub mod user;
pub mod article;

use thiserror::Error;
#[derive(Debug, Error, Clone)]
pub enum LoginError {
	#[error("账号/密码错误，请重新输入！")]
	WrongCredentials,

	#[error("账号已冻结!")]
	AccountFrozen,

	#[error("请输入管理员账号!")]
	AdminAccountRequired,
}
pub type LoginResult<T> = Result<T, LoginError>;


#[derive(Debug, Error, Clone)]
pub enum ArticleError {
	#[error("未找到文章")]
	ArticleNotFound,
}

pub type ArticleResult<T> = Result<T, ArticleError>;