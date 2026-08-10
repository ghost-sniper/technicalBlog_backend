use crate::models::article::{Article, ShortArticle};
use deadpool_postgres::Client;
use std::option::Option;

pub async fn get_article_by_id(client: &Client, id: i32) -> Option<Article> {
	let result = client
		.query_one(
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
		)
		.await
		.unwrap();
	Article::try_from(&result).ok()
}

pub async fn insert_article(client: &Client, article: &Article) -> Option<u64> {
	let result = client
		.execute(
			"INSERT INTO poetize.article (user_id, sort_id, title, content, cover)",
			&[&article.user_id, &article.article_title],
		)
		.await
		.unwrap();
	Some(result)
}

pub async fn get_all_articles(client: &Client) -> Vec<Article> {
	let result = client
		.query(
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
		)
		.await
		.unwrap();
	result
		.iter()
		.map(|row| Article::try_from(row).unwrap())
		.collect()
}

pub async fn get_articles_short_info(client: &Client) -> Vec<ShortArticle> {
	let result = client
		.query(
			"SELECT id, article_title,update_time,prospectus from poetize.article",
			&[],
		)
		.await
		.unwrap();
	result
		.iter()
		.map(|row| ShortArticle::try_from(row).unwrap())
		.collect()
}
