use crate::{models::article::{Article, ShortArticle}, select_row, select_rows};
use deadpool_postgres::Client;
use std::option::Option;

pub async fn get_article_by_id(client: &Client, id: i32) -> Option<Article> {
	select_row!(
		client,
		"select
	id,
	user_id,
	sort_id,
	label_id,
	article_cover,
	article_title,
	article_content,
	video_url,
	view_count,
	like_count,
	view_type,
	view_value,
	tips,
	recommend_status,
	comment_status,
	create_time,
	update_time,
	update_by,
	deleted,
	prospectus
from
	poetize.article
where
	id = $1",
		&[&id],
		Article
	)
}

pub async fn insert_article(client: &Client, article: &Article) -> Option<u64> {
	let result = client
		.execute(
			"INSERT INTO poetize.article (user_id, sort_id, article_title, article_content, article_cover) VALUES ($1, $2, $3, $4, $5)",
			&[
				&article.user_id,
				&article.sort_id,
				&article.article_title,
				&article.article_content,
				&article.article_cover,
			],
		)
		.await
		.unwrap();
	Some(result)
}

pub async fn get_all_articles(client: &Client) -> Vec<Article> {
	select_rows!(
		client,
		"select
	id,
	user_id,
	sort_id,
	label_id,
	article_cover,
	article_title,
	article_content,
	video_url,
	view_count,
	like_count,
	view_type,
	view_value,
	tips,
	recommend_status,
	comment_status,
	create_time,
	update_time,
	update_by,
	deleted,
	prospectus
from
	poetize.article",
		&[],
		Article
	)
}

pub async fn get_articles_short_info(client: &Client) -> Vec<ShortArticle> {
	select_rows!(
		client,
		"SELECT id, article_title,update_time,prospectus from poetize.article",
		&[],
		ShortArticle
	)
}
