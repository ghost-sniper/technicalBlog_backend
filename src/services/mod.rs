pub mod article;
pub mod article_images;
pub mod article_tags;
pub mod category;
pub mod sort;
pub mod tag;
pub mod user;

use thiserror::Error;
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
	#[error("查询分类失败，请稍后重试")]
	CategoryQueryFailed,
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
