use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;
#[derive(Serialize)]
pub struct TagResponse {
	tags: Vec<String>,
}
pub async fn get_all_tags() -> HttpResponse {
	let response = TagResponse {
		tags: vec![
			"Arch Linux".to_string(),
			"Php".to_string(),
			"Docker".to_string(),
		],
	};
	HttpResponse::Ok().status(StatusCode::OK).json(response)
}
