use crate::services::article_tags::{
	TAG_LOCK, best_match, list, resolve, valid_names,
};
use actix_identity::Identity;
use actix_web::{HttpResponse, http::StatusCode, web};
use deadpool_postgres::{Client, Pool};
pub fn admin_type() -> i16 {
	0
}
fn fail(status: StatusCode, message: &str) -> HttpResponse {
	HttpResponse::build(status).json(serde_json::json!({"message":message}))
}
fn internal(error: impl std::fmt::Display) -> HttpResponse {
	log::error!("标签管理失败: {error}");
	fail(StatusCode::INTERNAL_SERVER_ERROR, "标签管理暂时不可用")
}
async fn admin(
	identity: Option<Identity>,
	pool: &Pool,
) -> Result<(Client, i32, i16), HttpResponse> {
	let (c, id) = super::article_edit::actor(identity, pool).await?;
	let role = admin_type();
	let row = c
		.query_one("SELECT user_type FROM public.user WHERE id=$1", &[&id])
		.await
		.map_err(internal)?;
	if row.get::<_, i16>(0) != role {
		return Err(fail(StatusCode::FORBIDDEN, "此操作仅限管理员"));
	}
	Ok((c, id, role))
}
async fn lock(
	tx: &tokio_postgres::Transaction<'_>,
	id: i32,
	role: i16,
) -> Result<(), HttpResponse> {
	let row=tx.query_opt("SELECT id FROM public.user WHERE id=$1 AND user_type=$2 AND user_status=1 AND deleted=0 FOR SHARE",&[&id,&role]).await.map_err(internal)?;
	if row.is_none() {
		return Err(fail(StatusCode::FORBIDDEN, "管理员权限已失效"));
	}
	tx.query_one("SELECT pg_advisory_xact_lock($1)", &[&TAG_LOCK])
		.await
		.map_err(internal)?;
	Ok(())
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagName {
	pub tag_name: String,
}
pub async fn get(identity: Option<Identity>, pool: web::Data<Pool>) -> HttpResponse {
	let (c, _, _) = match admin(identity, &pool).await {
		Ok(v) => v,
		Err(e) => return e,
	};
	match c.query("SELECT t.id,t.tag_name,count(a.article_id) AS article_count FROM public.tag t LEFT JOIN public.article_tags a ON a.tag_id=t.id GROUP BY t.id,t.tag_name ORDER BY t.id",&[]).await{
 Ok(rows)=>HttpResponse::Ok().json(serde_json::json!({"tags":rows.iter().map(|r|serde_json::json!({"id":r.get::<_,i64>(0),"tag_name":r.get::<_,String>(1),"article_count":r.get::<_,i64>(2)})).collect::<Vec<_>>()})),Err(e)=>internal(e)}
}
pub async fn create(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	body: web::Json<TagName>,
) -> HttpResponse {
	let (mut c, id, role) = match admin(identity, &pool).await {
		Ok(v) => v,
		Err(e) => return e,
	};
	if !valid_names(&[body.tag_name.clone()]) {
		return fail(StatusCode::BAD_REQUEST, "标签名称需为 1 至 40 字");
	}
	let tx = match c.transaction().await {
		Ok(v) => v,
		Err(e) => return internal(e),
	};
	if let Err(e) = lock(&tx, id, role).await {
		return e;
	}
	let tags = match resolve(&tx, &[body.tag_name.clone()]).await {
		Ok(v) => v,
		Err(e) => return internal(e),
	};
	if let Err(e) = tx.commit().await {
		return internal(e);
	}
	HttpResponse::Ok()
		.json(serde_json::json!({"tag":tags[0],"message":"已使用匹配的标签"}))
}
pub async fn rename(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	id: web::Path<i64>,
	body: web::Json<TagName>,
) -> HttpResponse {
	let (mut c, uid, role) = match admin(identity, &pool).await {
		Ok(v) => v,
		Err(e) => return e,
	};
	let id = id.into_inner();
	if !valid_names(&[body.tag_name.clone()]) {
		return fail(StatusCode::BAD_REQUEST, "标签名称需为 1 至 40 字");
	}
	let tx = match c.transaction().await {
		Ok(v) => v,
		Err(e) => return internal(e),
	};
	if let Err(e) = lock(&tx, uid, role).await {
		return e;
	}
	let mut tags = match list(&*tx).await {
		Ok(v) => v,
		Err(e) => return internal(e),
	};
	if !tags.iter().any(|t| t.id == id) {
		return fail(StatusCode::NOT_FOUND, "标签不存在");
	}
	tags.retain(|t| t.id != id);
	if let Some(tag) = best_match(&body.tag_name, &tags) {
		return fail(
			StatusCode::CONFLICT,
			&format!("已存在相似标签「{}」，请使用该标签", tag.tag_name),
		);
	}
	if let Err(e) = tx
		.execute(
			"UPDATE public.tag SET tag_name=$1 WHERE id=$2",
			&[&body.tag_name.trim(), &id],
		)
		.await
	{
		return internal(e);
	}
	if let Err(e) = tx.commit().await {
		return internal(e);
	}
	HttpResponse::Ok().json(serde_json::json!({"message":"标签已更新"}))
}
pub async fn delete(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	id: web::Path<i64>,
) -> HttpResponse {
	let (mut c, uid, role) = match admin(identity, &pool).await {
		Ok(v) => v,
		Err(e) => return e,
	};
	let id = id.into_inner();
	let tx = match c.transaction().await {
		Ok(v) => v,
		Err(e) => return internal(e),
	};
	if let Err(e) = lock(&tx, uid, role).await {
		return e;
	}
	let count = match tx
		.query_one(
			"SELECT count(*) FROM public.article_tags WHERE tag_id=$1",
			&[&id],
		)
		.await
	{
		Ok(r) => r.get::<_, i64>(0),
		Err(e) => return internal(e),
	};
	if count > 0 {
		return fail(
			StatusCode::CONFLICT,
			"标签仍被文章使用，请先移除文章中的关联",
		);
	}
	match tx
		.execute("DELETE FROM public.tag WHERE id=$1", &[&id])
		.await
	{
		Ok(0) => return fail(StatusCode::NOT_FOUND, "标签不存在"),
		Ok(_) => {}
		Err(e) => return internal(e),
	}
	if let Err(e) = tx.commit().await {
		return internal(e);
	}
	HttpResponse::Ok().json(serde_json::json!({"message":"标签已删除"}))
}

#[cfg(test)]
mod tests {
	use super::*;
	#[actix_web::test]
	async fn anonymous_management_requests_are_rejected() {
		let pool = crate::db::create_database_pool(
			"127.0.0.1".into(),
			"1".into(),
			"unused".into(),
			"unused".into(),
			"".into(),
		);
		assert_eq!(
			admin(None, &pool).await.err().unwrap().status(),
			StatusCode::UNAUTHORIZED
		);
		assert_eq!(admin_type(), 0);
	}
	#[actix_web::test]
	#[ignore = "requires PostgreSQL tunnel; fixture transaction rolls back"]
	async fn live_admin_role_and_frozen_account_checks() {
		dotenvy::dotenv().ok();
		let pool = crate::db::create_database_pool(
			std::env::var("DATABASE_HOST").unwrap(),
			std::env::var("DATABASE_PORT").unwrap(),
			std::env::var("DATABASE_NAME").unwrap(),
			std::env::var("DATABASE_USER").unwrap(),
			std::env::var("DATABASE_USER_PASSWORD").unwrap(),
		);
		let mut c = pool.get().await.unwrap();
		let tx = c.transaction().await.unwrap();
		let name =
			format!("adm-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
		let id:i32=tx.query_one("INSERT INTO public.user(username,password,user_type) VALUES($1,'test-only',2) RETURNING id",&[&name]).await.unwrap().get(0);
		assert_eq!(
			lock(&tx, id, admin_type()).await.err().unwrap().status(),
			StatusCode::FORBIDDEN
		);
		tx.execute("UPDATE public.user SET user_type=0 WHERE id=$1", &[&id])
			.await
			.unwrap();
		assert!(lock(&tx, id, admin_type()).await.is_ok());
		tx.execute("UPDATE public.user SET user_status=0 WHERE id=$1", &[&id])
			.await
			.unwrap();
		assert_eq!(
			lock(&tx, id, admin_type()).await.err().unwrap().status(),
			StatusCode::FORBIDDEN
		);
		tx.rollback().await.unwrap();
	}
}
