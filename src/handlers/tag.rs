use actix_web::{HttpResponse, web};
use deadpool_postgres::Pool;
use serde::Serialize;

use crate::models::tag::{Tag, TagRequest};
#[derive(Serialize)]
pub struct TagResponse {
	tags: Vec<Tag>,
	count: usize,
}
pub async fn get_all_tags(pool: web::Data<Pool>) -> HttpResponse {
	let client = match pool.get().await {
		Ok(client) => client,
		Err(e) => {
			eprintln!("Failed to get Database connection: {}", e);
			return HttpResponse::InternalServerError().finish();
		}
	};
	let rows = client
		.query("SELECT id ,tag_name, status From tag", &[])
		.await;
	return match rows {
		Ok(rows) => {
			let tags: Vec<Tag> = rows
				.iter()
				.map(|row| {
					println!("{:?}", row.try_get::<_, i64>(0));
					println!("{:?}", row.try_get::<_, &str>("tag_name"));
					println!("{:?}", row.try_get::<_, i32>(2));
					Tag::new(
						row.get::<_, i64>(0),
						row.get::<_, String>(1),
						row.get::<_, i32>(2),
					)
				})
				.collect();
			println!("{:?}", tags);
			let response = TagResponse {
				count: tags.len(),
				tags: tags,
			};
			HttpResponse::Ok().json(response)
		}
		Err(e) => {
			eprintln!("Query error: {}", e);
			HttpResponse::InternalServerError().finish()
		}
	};
}

pub async fn insert_tag(
	pool: web::Data<Pool>,
	request: web::Json<TagRequest>,
) -> HttpResponse {
	let client = match pool.get().await {
		Ok(client) => client,
		Err(e) => {
			eprintln!("Failed to get Database connection: {}", e);
			return HttpResponse::InternalServerError().finish();
		}
	};
	let affected_rows = client
		.execute(
			"Insert into tag(tag_name,status) values($1,$2)",
			&[&request.tag_name, &request.status],
		)
		.await
		.expect("Failed to insert into tag");
	if affected_rows == 1 {
		HttpResponse::Ok().body("Tag inserted successfully")
	} else {
		HttpResponse::InternalServerError().body("Insert to tag failed")
	}
}
