use crate::handlers::get_db_client;
use crate::models::tag::{Tag, TagRequest};
use crate::services::tag::{create_tag, get_tag_list};
use actix_web::{HttpResponse, web};
use deadpool_postgres::Pool;
use serde::Serialize;

#[derive(Serialize)]
pub struct TagResponse {
	tags: Vec<Tag>,
	count: usize,
}
pub async fn get_all_tags(pool: web::Data<Pool>) -> HttpResponse {
	let client = get_db_client(pool).await.unwrap();
	let result = get_tag_list(&client).await;
	match result {
		Ok(tags) => HttpResponse::Ok().json(TagResponse {
			count: tags.len(),
			tags,
		}),
		Err(e) => HttpResponse::Ok().body(e.to_string()),
	}
}

pub async fn insert_tag(
	pool: web::Data<Pool>,
	request: web::Json<TagRequest>,
) -> HttpResponse {
	let client = get_db_client(pool).await.unwrap();
	let result = create_tag(&client, &*request.tag_name, request.status).await;
	match result {
		Ok(_) => HttpResponse::Ok().body("Tag inserted successfully"),
		Err(e) => HttpResponse::Ok().body(e.to_string()),
	}
}
