pub mod sort;
pub mod user;
pub mod article;
pub mod tag;
pub mod category;

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


#[derive(Debug, Error, Clone)]
pub enum TagError {
	#[error("未查询到标签")]
	TagNotFound,
	#[error("新增标签失败")]
	TagCreationFailed,
	#[error("修改标签失败")]
	TagModificationFailed,
	#[error("删除标签失败")]
	TagDeleteFailed,
}

pub type TagResult<T> = Result<T, TagError>;


#[derive(Debug, Error, Clone)]
pub enum CategoryError {
	#[error("未查询到分类")]
	CategoryNotFound,
	#[error("新增分类失败")]
	CategoryCreationFailed,
	#[error("修改分类失败")]
	CategoryModificationFailed,
	#[error("删除分类失败")]
	CategoryDeleteFailed,
}

pub type CategoryResult<T> = Result<T, CategoryError>;