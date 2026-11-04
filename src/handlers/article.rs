use crate::dao::article::get_article_by_id;
use crate::handlers::get_db_client;
use crate::handlers::presentation::UnifiedResponse;
use actix_web::{HttpResponse, web};
use deadpool_postgres::Pool;

pub async fn get_article(
	pool: web::Data<Pool>,
	id: web::Path<String>,
) -> HttpResponse {
	let client = get_db_client(pool).await.unwrap();
	let article_id = id.into_inner();
	let rows = get_article_by_id(&client, article_id.parse::<i32>().unwrap()).await;
	if rows.is_none() {
		HttpResponse::Ok().json(UnifiedResponse {
			status: "success".to_string(),
			code: 200,
			message: None::<String>,
		})
	} else {
		let article = rows.unwrap();
		let tags =
			match crate::services::article_tags::for_article(&**client, article.id)
				.await
			{
				Ok(tags) => tags,
				Err(error) => {
					log::error!("文章标签查询失败: {error}");
					return HttpResponse::InternalServerError()
						.json(serde_json::json!({"message":"文章标签查询失败"}));
				}
			};
		let mut article = serde_json::to_value(article).unwrap();
		article["tags"] = serde_json::to_value(tags).unwrap();
		HttpResponse::Ok().json(
			serde_json::json!({"status":"success","code":200,"message":article}),
		)
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
		)
		.await
	}
	.await;
	match result {
        Ok((total, articles)) => HttpResponse::Ok().json(serde_json::json!({
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
