use actix_web::{HttpResponse, web};
use deadpool_postgres::Pool;

use crate::dao::sort::query_all_label;

pub async fn list_label(pool: web::Data<Pool>) -> HttpResponse {
	println!("正在查询标签");
	let client = match pool.get().await {
		Ok(client) => client,
		Err(e) => {
			eprintln!("Failed to get DataBase connection: {}", e);
			return HttpResponse::InternalServerError().finish();
		}
	};
	let result = query_all_label(&client).await;
	if let None = result {
		print!("Not Found");
		return HttpResponse::Ok().finish();
	} else {
		println!("Found");
		let s = result.unwrap();
		s.iter().for_each(|i| {
			println!("{:?}", i);
		});
		return HttpResponse::Ok().json(s);
	}
	// return HttpResponse::Ok().finish();
}
