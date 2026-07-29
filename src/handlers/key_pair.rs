use crate::dao;
use actix_web::HttpResponse;
use actix_web::web;
use deadpool_postgres::Pool;
pub async fn get_public_key(pool: web::Data<Pool>) -> HttpResponse {
	let client = pool.get().await.unwrap();
	let public_key = dao::key_pair::get_public_key(&client).await.unwrap();
	HttpResponse::Ok().body(public_key)
}
