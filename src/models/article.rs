use blog::TryFromRow;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Deserialize, Debug, Serialize, Clone,TryFromRow)]
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
	pub tips: Option<String>,
	pub recommend_status: i16,
	pub comment_status: i16,
	pub create_time: Option<NaiveDateTime>,
	pub update_time: Option<NaiveDateTime>,
	pub update_by: Option<String>,
	pub deleted: i16,
	pub prospectus: Option<String>,
}

#[derive(Deserialize, Debug, Serialize, Clone, TryFromRow)]
pub struct ShortArticle {
	pub id: i32,
	pub article_title: String,
	pub update_time: Option<NaiveDateTime>,
	pub prospectus: Option<String>,
}