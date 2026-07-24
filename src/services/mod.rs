pub mod user;
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
