use deadpool_postgres::Client;
use crate::models::article::Article;
use crate::services::{ArticleError, ArticleResult};
use crate::dao::article::get_article_by_id;
pub async fn get_article(client: &Client, id: i32) -> ArticleResult<Article> {
    let result = get_article_by_id(client, id).await;
    if result.is_none() {
        Err(ArticleError::ArticleNotFound)
    } else {
        Ok(result.unwrap())
    }
}