use crate::handlers::get_db_client;
use crate::models::category::Category;
use crate::services::category::get_category_list;
use actix_web::HttpResponse;
use actix_web::web;
use deadpool_postgres::Pool;
use serde::Serialize;

#[derive(Serialize)]
pub struct CategoriesResponse {
	categories: Vec<Category>,
	count: usize,
}
pub async fn get_all_categories(pool: web::Data<Pool>) -> HttpResponse {
	let client = get_db_client(pool)
		.await
		.expect("Failed to get database client");

	let result = get_category_list(&client).await;
	match result {
		Ok(categories) => HttpResponse::Ok().json(CategoriesResponse {
			count: categories.len(),
			categories,
		}),
		Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
	}
}
