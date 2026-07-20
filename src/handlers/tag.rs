use actix_web::{HttpResponse, web};
use deadpool_postgres::Pool;
use serde::Serialize;

use crate::models::tag::Tag;
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
