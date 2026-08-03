use crate::dao::article::get_article_by_id;
use actix_web::{HttpResponse, web};
use deadpool_postgres::Pool;

pub async fn get_article(
	pool: web::Data<Pool>,
	id: web::Path<String>,
) -> HttpResponse {
	let client = match pool.get().await {
		Ok(client) => client,
		Err(e) => {
            eprintln!("failed to get db client: {}", e);
			return HttpResponse::InternalServerError().finish();
		}
	};
	let article_id = id.into_inner();
	let rows = get_article_by_id(&client, article_id.parse::<i32>().unwrap()).await;
	if rows.is_none() {
		HttpResponse::InternalServerError().finish()
	} else {
		HttpResponse::Ok().json(rows.unwrap())
	}
}

