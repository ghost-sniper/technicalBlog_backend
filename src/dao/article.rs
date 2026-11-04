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
	public.article
where
	id = $1",
		&[&id],
		Article
	)
}

pub async fn insert_article(client: &Client, article: &Article) -> Option<u64> {
	let result = client
		.execute(
			"INSERT INTO public.article (user_id, sort_id, article_title, article_content, article_cover) VALUES ($1, $2, $3, $4, $5)",
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
	public.article",
		&[],
		Article
	)
}

pub async fn get_articles_short_info(client: &Client) -> Vec<ShortArticle> {
	select_rows!(
		client,
		"SELECT a.id, a.article_title, a.update_time, a.prospectus,
			ARRAY(SELECT link.tag_id FROM public.article_tags link
				WHERE link.article_id = a.id ORDER BY link.tag_id) AS tag_ids
		FROM public.article a",
		&[],
		ShortArticle
	)
}

pub async fn get_article_page(
	client: &Client,
	page: i64,
	page_size: i64,
	keyword: &str,
	tag_id: Option<i64>,
) -> anyhow::Result<(i64, Vec<ShortArticle>)> {
	let offset = (page - 1) * page_size;
	let pattern = format!("%{}%", keyword.replace('!', "!!").replace('%', "!%").replace('_', "!_"));
	let row = client.query_one(
		"WITH matched AS NOT MATERIALIZED (
			SELECT a.id, a.update_time
			FROM public.article a
			WHERE a.deleted=0
				AND ($1 = '%%' OR lower(a.article_title || ' ' || coalesce(a.prospectus,'')
					|| ' ' || coalesce(a.article_content,'')) LIKE lower($1) ESCAPE '!')
				AND ($2::bigint IS NULL OR EXISTS (
					SELECT 1 FROM public.article_tags link WHERE link.article_id=a.id AND link.tag_id=$2
				))
		), page_ids AS MATERIALIZED (
			SELECT * FROM matched ORDER BY update_time DESC NULLS LAST, id DESC LIMIT $3 OFFSET $4
		), page_rows AS (
			SELECT a.id, a.article_title, a.update_time, a.prospectus,
				ARRAY(SELECT link.tag_id FROM public.article_tags link
					WHERE link.article_id=a.id ORDER BY link.tag_id) AS tag_ids
			FROM page_ids p JOIN public.article a ON a.id=p.id
		)
		SELECT (SELECT count(*) FROM matched) AS total,
			coalesce((SELECT jsonb_agg(p ORDER BY p.update_time DESC NULLS LAST, p.id DESC)
				FROM page_rows p), '[]'::jsonb) AS articles",
		&[&pattern, &tag_id, &page_size, &offset],
	).await?;
	Ok((row.try_get("total")?, serde_json::from_value(row.try_get("articles")?)?))
}
