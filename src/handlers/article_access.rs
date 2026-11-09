use actix_identity::Identity;
use actix_web::{HttpResponse, http::StatusCode};
use tokio_postgres::GenericClient;

pub fn viewer(identity: Option<Identity>) -> Option<i32> {
	identity.and_then(|identity| identity.id().ok()?.parse().ok())
}
pub fn fail(status: StatusCode, message: &str) -> HttpResponse {
	HttpResponse::build(status).json(serde_json::json!({"message": message}))
}
pub fn internal(error: impl std::fmt::Display) -> HttpResponse {
	log::error!("文章服务失败: {error}");
	fail(
		StatusCode::INTERNAL_SERVER_ERROR,
		"文章服务暂时不可用，请稍后重试",
	)
}
// Hold the article lock until response data is assembled / the mutation commits.
// Writers acquire the user lock first, matching the article editor's lock order.
pub async fn readable<C: GenericClient + Sync>(
	client: &C,
	id: i32,
	viewer: Option<i32>,
	write: bool,
) -> Result<tokio_postgres::Row, HttpResponse> {
	let sql = format!(
		"SELECT a.user_id,a.comment_status,public.can_read_article(a,$2) AS allowed FROM public.article a WHERE a.id=$1 AND a.deleted=0 FOR {}",
		if write { "UPDATE" } else { "SHARE" }
	);
	let row = client
		.query_opt(&sql, &[&id, &viewer])
		.await
		.map_err(internal)?
		.ok_or_else(|| fail(StatusCode::NOT_FOUND, "文章不存在或暂不可访问"))?;
	if !row.get::<_, bool>("allowed") {
		return Err(fail(StatusCode::NOT_FOUND, "文章不存在或暂不可访问"));
	}
	Ok(row)
}
