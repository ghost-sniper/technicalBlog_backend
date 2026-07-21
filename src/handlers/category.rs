use crate::models::category::Category;
use actix_web::HttpResponse;
use actix_web::web;
use chrono::NaiveDateTime;
use deadpool_postgres::Pool;
use serde::Serialize;
#[derive(Serialize)]
pub struct CategoriesResponse {
	categories: Vec<Category>,
	count: usize,
}
pub async fn get_all_categories(pool: web::Data<Pool>) -> HttpResponse {
	let client = match pool.get().await {
		Ok(client) => client,
		Err(_) => {
			return HttpResponse::InternalServerError()
				.body("Failed to get database client");
		}
	};
	let rows = client
		.query(
			"SELECT id,name,description,is_deleted,created_at,updated_at FROM article_categories",
			&[],
		)
		.await;
	return match rows {
		Ok(rows) => {
			let categories: Vec<Category> = rows
				.iter()
				.map(|row| {
					Category::new(
						row.get::<_, i64>(0),
						row.get::<_, String>(1),
						row.get::<_, String>(2),
						row.get::<_, i16>(3),
						row.get::<_, NaiveDateTime>(4),
						row.get::<_, NaiveDateTime>(5),
					)
				})
				.collect();
			let response = CategoriesResponse {
				count: categories.len(),
				categories: categories,
			};
			HttpResponse::Ok().json(response)
		}
		Err(_) => {
			HttpResponse::InternalServerError().body("Failed to get categories")
		}
	};
}
