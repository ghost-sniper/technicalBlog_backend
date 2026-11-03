use super::article_edit::actor;
use crate::services::article_images::{ArticleImageStorage, PostgresArticleImages};
use actix_identity::Identity;
use actix_web::{
	HttpResponse,
	http::{StatusCode, header},
	web,
};
use deadpool_postgres::Pool;

pub const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;

fn error(status: StatusCode, message: &str) -> HttpResponse {
	HttpResponse::build(status).json(serde_json::json!({ "message": message }))
}

fn storage_error(cause: impl std::fmt::Display) -> HttpResponse {
	log::error!("文章图片存取失败: {cause}");
	error(StatusCode::INTERNAL_SERVER_ERROR, "图片服务暂时不可用")
}

fn image_type(bytes: &[u8]) -> Option<&'static str> {
	if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
		Some("image/png")
	} else if bytes.starts_with(b"\xff\xd8\xff") {
		Some("image/jpeg")
	} else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
		Some("image/gif")
	} else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
		Some("image/webp")
	} else {
		None
	}
}

pub async fn upload(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	article_id: web::Path<i32>,
	body: web::Bytes,
) -> HttpResponse {
	let (client, author_id) = match actor(identity, &pool).await {
		Ok(value) => value,
		Err(response) => return response,
	};
	let Some(content_type) = image_type(&body) else {
		return error(
			StatusCode::UNSUPPORTED_MEDIA_TYPE,
			"请选择 PNG、JPEG、GIF 或 WebP 图片",
		);
	};
	match PostgresArticleImages(&client)
		.put(*article_id, author_id, content_type, &body)
		.await
	{
		Ok(Some(id)) => HttpResponse::Created().json(serde_json::json!({
			"url": format!("/api/article-images/{id}")
		})),
		Ok(None) => error(StatusCode::FORBIDDEN, "你没有这篇文章的编辑权限"),
		Err(cause) => storage_error(cause),
	}
}

pub async fn read(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	id: web::Path<i64>,
) -> HttpResponse {
	let client = match pool.get().await {
		Ok(client) => client,
		Err(cause) => return storage_error(cause),
	};
	let viewer_id =
		identity.and_then(|identity| identity.id().ok()?.parse::<i32>().ok());
	match PostgresArticleImages(&client).get(*id, viewer_id).await {
		Ok(Some(image)) => HttpResponse::Ok()
			.content_type(image.content_type)
			.insert_header((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
			.insert_header((header::CACHE_CONTROL, "private, no-store"))
			.body(image.content),
		Ok(None) => error(StatusCode::NOT_FOUND, "图片不存在"),
		Err(cause) => storage_error(cause),
	}
}
