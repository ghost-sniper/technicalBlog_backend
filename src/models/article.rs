use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Deserialize,Debug,Serialize,Clone)]
pub struct Article {
    pub id: i32,
    pub user_id: i32,
    pub sort_id: i32,
    pub label_id: i32,
    pub article_cover: Option<String>,
    pub article_title: String,
    pub article_content: String,
    pub video_url: Option<String>,
    pub view_count: i32,
    pub like_count: i32,
    pub view_type: String,
    pub view_value: Option<String>,
    pub recommend_status: i16,
    pub comment_status: i16,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
    pub update_by: Option<String>,
    pub deleted: bool
}
impl TryFrom<&Row>  for Article {
    type Error = tokio_postgres::Error;
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Article{
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            sort_id: row.try_get("sort_id")?,
            label_id: row.try_get("label_id")?,
            article_cover: row.try_get("article_cover")?,
            article_title: row.try_get("article_title")?,
            article_content: row.try_get("article_content")?,
            video_url: row.try_get("video_url")?,
            view_count: row.try_get("view_count")?,
            like_count: row.try_get("like_count")?,
            view_type: row.try_get("view_type")?,
            view_value: row.try_get("view_value")?,
            recommend_status: row.try_get("recommend_status")?,
            comment_status: row.try_get("comment_status")?,
            create_time: row.try_get("create_time")?,
            update_time: row.try_get("update_time")?,
            update_by: row.try_get("update_by")?,
            deleted: row.try_get("deleted")?,
        })
    }
}