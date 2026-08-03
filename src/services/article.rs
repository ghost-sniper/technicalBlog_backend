use crate::dao::article::{get_all_articles, get_article_by_id};
use crate::models::article::Article;
use crate::services::{ArticleError, ArticleResult};
use deadpool_postgres::Client;
pub async fn get_article(client: &Client, id: i32) -> ArticleResult<Article> {
	let result = get_article_by_id(client, id).await;
	if result.is_none() {
		Err(ArticleError::ArticleNotFound)
	} else {
		Ok(result.unwrap())
	}
}

pub async fn get_articles(
	client: &Client,
) -> ArticleResult<Vec<Article>> {
	let result = get_all_articles(client).await;
	if result.is_empty() {
		Err(ArticleError::ArticleNotFound)
	} else {
		Ok(result)
	}
}
