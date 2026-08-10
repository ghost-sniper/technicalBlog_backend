use crate::dao::article::get_article_by_id;
use crate::handlers::get_db_client;
use crate::handlers::presentation::UnifiedResponse;
use crate::handlers::presentation::article::ShortArticleResponse;
use crate::services::article::get_short_article_list;
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
		HttpResponse::Ok().json(UnifiedResponse {
			status: "success".to_string(),
			code: 200,
			message: rows,
		})
	}
}

pub async fn get_all_short_article(pool: web::Data<Pool>) -> HttpResponse {
	let client = get_db_client(pool).await.unwrap();
	let rows = get_short_article_list(&client).await;

	if rows.is_err() {
		HttpResponse::Ok().json(UnifiedResponse {
			status: rows.err().unwrap().to_string(),
			code: 1002,
			message: None::<String>,
		})
	} else {
		HttpResponse::Ok().json(UnifiedResponse {
			status: "success".to_string(),
			code: 200,
			message: Some(ShortArticleResponse {
				article_list: rows.clone().unwrap(),
				count: rows.ok().unwrap().len() as i32,
			}),
		})
	}
}
