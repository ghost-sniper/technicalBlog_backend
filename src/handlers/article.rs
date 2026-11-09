use super::article_access::{internal, readable, viewer};
use actix_identity::Identity;
use actix_web::{HttpResponse, web};
use deadpool_postgres::Pool;

pub async fn get_article(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	id: web::Path<i32>,
) -> HttpResponse {
	let mut client = match pool.get().await {
		Ok(c) => c,
		Err(e) => return internal(e),
	};
	let tx = match client.transaction().await {
		Ok(tx) => tx,
		Err(e) => return internal(e),
	};
	if let Err(e) = readable(&*tx, *id, viewer(identity), false).await {
		return e;
	}
	let result = async {
		let row = tx
			.query_one(
				"SELECT to_jsonb(a) - 'view_value' AS article FROM public.article a WHERE id=$1",
				&[&*id],
			)
			.await?;
		let mut article: serde_json::Value = row.get("article");
		article["tags"] = serde_json::to_value(
			crate::services::article_tags::for_article(&*tx, *id).await?,
		)?;
		Ok::<_, anyhow::Error>(article)
	}
	.await;
	match result {
		Ok(article) => HttpResponse::Ok()
			.insert_header(("Cache-Control", "private, no-store"))
			.json(
				serde_json::json!({"status":"success","code":200,"message":article}),
			),
		Err(e) => internal(e),
	}
}

#[derive(serde::Deserialize)]
pub struct ArticleListQuery {
	#[serde(default = "first_page")]
	page: i64,
	#[serde(default = "default_page_size")]
	page_size: i64,
	#[serde(default)]
	q: String,
	tag_id: Option<i64>,
}
fn first_page() -> i64 {
	1
}
fn default_page_size() -> i64 {
	10
}

pub async fn get_all_short_article(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	query: web::Query<ArticleListQuery>,
) -> HttpResponse {
	if query.page < 1
		|| !(1..=100).contains(&query.page_size)
		|| query
			.page
			.checked_sub(1)
			.and_then(|page| page.checked_mul(query.page_size))
			.is_none()
		|| query.q.chars().count() > 255
		|| query.tag_id.is_some_and(|id| id < 1)
	{
		return HttpResponse::BadRequest().json(serde_json::json!({
            "code":40001,"message":"分页参数无效，页码须大于 0，每页 1 至 100 篇，关键词最多 255 字"
        }));
	}
	let result = async {
		let client = pool.get().await?;
		crate::dao::article::get_article_page(
			&client,
			query.page,
			query.page_size,
			query.q.trim(),
			query.tag_id,
			viewer(identity),
		)
		.await
	}
	.await;
	match result {
        Ok((total, articles)) => HttpResponse::Ok().insert_header(("Cache-Control", "private, no-store")).json(serde_json::json!({
            "status":"success","code":200,"message":{
                "article_list":articles,"count":total,"page":query.page,"page_size":query.page_size
            }
        })),
        Err(error) => {
            log::error!("文章列表查询失败: {error}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "code":50001,"message":"文章列表暂时不可用，请稍后重试"
            }))
        }
    }
}
