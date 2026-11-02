use crate::handlers::get_db_client;

use actix_web::{HttpResponse, web};
use deadpool_postgres::Pool;

#[derive(serde::Deserialize)]
pub struct TagSearch {
	pub q: Option<String>,
}
pub async fn get_all_tags(
	pool: web::Data<Pool>,
	search: web::Query<TagSearch>,
) -> HttpResponse {
	let client = match get_db_client(pool).await {
		Ok(client) => client,
		Err(_) => {
			return HttpResponse::InternalServerError()
				.json(serde_json::json!({"message":"标签服务暂时不可用"}));
		}
	};
	match crate::services::article_tags::list(&**client).await {
		Ok(tags) => {
			let q = search.q.as_deref().unwrap_or("");
			if q.chars().count() > 40 {
				return HttpResponse::BadRequest()
					.json(serde_json::json!({"message":"搜索标签最多 40 字"}));
			}
			let mut suggestions: Vec<_> = tags
				.iter()
				.filter_map(|tag| {
					let score =
						crate::services::article_tags::similarity(q, &tag.tag_name);
					let normalized = crate::services::article_tags::normalize(q);
					(normalized.is_empty()
						|| score >= 0.5
						|| crate::services::article_tags::normalize(&tag.tag_name)
							.contains(&normalized))
					.then_some(
						serde_json::json!({"id":tag.id,"tag_name":tag.tag_name,"similarity":score}),
					)
				})
				.collect();
			suggestions.sort_by(|a, b| {
				b["similarity"]
					.as_f64()
					.unwrap()
					.total_cmp(&a["similarity"].as_f64().unwrap())
			});
			HttpResponse::Ok().json(
				serde_json::json!({"count":tags.len(),"tags":tags,"suggestions":suggestions}),
			)
		}
		Err(error) => {
			log::error!("标签查询失败: {error}");
			HttpResponse::InternalServerError()
				.json(serde_json::json!({"message":"标签查询失败"}))
		}
	}
}
