use actix_web::HttpResponse;
use serde::Serialize;
#[derive(Serialize)]
pub struct CategoriesResponse {
	categories: Vec<String>,
}
pub async fn get_all_categories() -> HttpResponse {
	let response = CategoriesResponse {
		categories: vec!["AI".to_string(), "".to_string(), "".to_string()],
	};
	let body = serde_json::to_string(&response).unwrap();
	HttpResponse::Ok().body(body)
}
