use serde::{Deserialize, Serialize};
use crate::models::article::ShortArticle;

#[derive(Debug, Clone,Serialize,Deserialize)]
pub struct ShortArticleResponse{
    pub count: i32,
    pub article_list: Vec<ShortArticle>,
}