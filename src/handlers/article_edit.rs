use actix_identity::Identity;
use actix_web::{HttpResponse, http::StatusCode, web};
use chrono::NaiveDateTime;
use deadpool_postgres::{Client, Pool};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct EditableArticle {
	pub view_type: String,
	pub comment_status: i16,
	#[serde(default)]
	pub tags: Vec<crate::models::tag::Tag>,
	pub id: i32,
	pub article_title: String,
	pub article_content: String,
	pub prospectus: Option<String>,
	pub update_time: Option<NaiveDateTime>,
}
impl TryFrom<&tokio_postgres::Row> for EditableArticle {
	type Error = tokio_postgres::Error;
	fn try_from(row: &tokio_postgres::Row) -> Result<Self, Self::Error> {
		Ok(Self {
			view_type: row.try_get("view_type")?,
			comment_status: row.try_get("comment_status")?,
			id: row.try_get("id")?,
			article_title: row.try_get("article_title")?,
			article_content: row.try_get("article_content")?,
			prospectus: row.try_get("prospectus")?,
			update_time: row.try_get("update_time")?,
			tags: vec![],
		})
	}
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditRequest {
	pub view_type: Option<String>,
	pub comment_status: Option<i16>,
	#[serde(default)]
	pub tags: Option<Vec<String>>,
	pub article_title: String,
	pub article_content: String,
	pub prospectus: Option<String>,
	pub expected_update_time: Option<NaiveDateTime>,
}
fn failure(status: StatusCode, code: i32, message: &str) -> HttpResponse {
	HttpResponse::build(status)
		.json(serde_json::json!({"code":code,"message":message}))
}
fn internal(error: impl std::fmt::Display) -> HttpResponse {
	log::error!("文章编辑失败: {error}");
	failure(
		StatusCode::INTERNAL_SERVER_ERROR,
		50001,
		"文章服务暂时不可用，请稍后重试",
	)
}
pub(super) async fn actor(
	identity: Option<Identity>,
	pool: &Pool,
) -> Result<(Client, i32), HttpResponse> {
	let identity = identity
		.ok_or_else(|| failure(StatusCode::UNAUTHORIZED, 40100, "请先登录"))?;
	let id = identity
		.id()
		.ok()
		.and_then(|id| id.parse::<i32>().ok())
		.ok_or_else(|| failure(StatusCode::UNAUTHORIZED, 40100, "登录已失效"))?;
	let client = pool.get().await.map_err(internal)?;
	let user = client
		.query_opt(
			"SELECT user_status,deleted FROM public.user WHERE id=$1",
			&[&id],
		)
		.await
		.map_err(internal)?;
	if !user.is_some_and(|row| {
		row.get::<_, i16>("user_status") == 1 && row.get::<_, i16>("deleted") == 0
	}) {
		identity.logout();
		return Err(failure(StatusCode::FORBIDDEN, 40301, "账号已被冻结或删除"));
	}
	Ok((client, id))
}
fn validate(request: &EditRequest) -> bool {
	request
		.view_type
		.as_deref()
		.is_none_or(|v| matches!(v, "public" | "authenticated" | "private"))
		&& request.comment_status.is_none_or(|v| v == 0 || v == 1)
		&& request
			.tags
			.as_ref()
			.is_none_or(|names| crate::services::article_tags::valid_names(names))
		&& !request.article_title.trim().is_empty()
		&& request.article_title.chars().count() <= 255
		&& !request.article_content.trim().is_empty()
		&& request.article_content.len() <= 1_500_000
		&& request
			.prospectus
			.as_ref()
			.is_none_or(|value| value.chars().count() <= 255)
}
pub async fn get_editable(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	id: web::Path<i32>,
) -> HttpResponse {
	let (client, actor) = match actor(identity, &pool).await {
		Ok(value) => value,
		Err(response) => return response,
	};
	match client.query_opt("SELECT id,user_id,article_title,article_content,prospectus,update_time,view_type,comment_status FROM public.article WHERE id=$1 AND deleted=0",&[&id.into_inner()]).await {
        Ok(Some(row)) if row.get::<_,i32>("user_id") == actor => match EditableArticle::try_from(&row){Ok(mut article)=>{ article.tags=match crate::services::article_tags::for_article(&**client,article.id).await {Ok(tags)=>tags,Err(error)=>return internal(error)}; HttpResponse::Ok().json(article) },Err(error)=>internal(error)},
        Ok(Some(_))=>failure(StatusCode::FORBIDDEN,40302,"你没有这篇文章的编辑权限"),
        Ok(None)=>failure(StatusCode::NOT_FOUND,40401,"文章不存在或已删除"),
        Err(error)=>internal(error),
    }
}
pub async fn save(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	id: web::Path<i32>,
	request: web::Json<EditRequest>,
) -> HttpResponse {
	let (mut client, actor) = match actor(identity, &pool).await {
		Ok(value) => value,
		Err(response) => return response,
	};
	if !validate(&request) {
		return failure(
			StatusCode::BAD_REQUEST,
			40001,
			"请检查访问范围和评论开关，填写标题和 Markdown 正文；标题、摘要最多 255 字，正文最多 1.5 MB",
		);
	}
	let tx = match client.transaction().await {
		Ok(tx) => tx,
		Err(error) => return internal(error),
	};
	let article = match persist(&tx, id.into_inner(), actor, &request).await {
		Ok(article) => article,
		Err(response) => return response,
	};
	if let Err(error) = tx.commit().await {
		return internal(error);
	}
	HttpResponse::Ok().json(article)
}
async fn persist(
	tx: &tokio_postgres::Transaction<'_>,
	id: i32,
	actor: i32,
	request: &EditRequest,
) -> Result<EditableArticle, HttpResponse> {
	let user=tx.query_opt("SELECT id FROM public.user WHERE id=$1 AND user_status=1 AND deleted=0 FOR SHARE",&[&actor]).await.map_err(internal)?;
	if user.is_none() {
		return Err(failure(StatusCode::FORBIDDEN, 40301, "账号已被冻结或删除"));
	}
	let row=tx.query_opt("SELECT user_id,update_time FROM public.article WHERE id=$1 AND deleted=0 FOR UPDATE",&[&id]).await.map_err(internal)?.ok_or_else(||failure(StatusCode::NOT_FOUND,40401,"文章不存在或已删除"))?;
	if row.get::<_, i32>("user_id") != actor {
		return Err(failure(
			StatusCode::FORBIDDEN,
			40302,
			"你没有这篇文章的编辑权限",
		));
	}
	if row.get::<_, Option<NaiveDateTime>>("update_time")
		!= request.expected_update_time
	{
		return Err(failure(
			StatusCode::CONFLICT,
			40901,
			"文章已被更新，请先备份当前内容，再重新加载",
		));
	}
	let editor = actor.to_string();
	let row=tx.query_one("UPDATE public.article SET article_title=$1,article_content=$2,prospectus=$3,update_time=clock_timestamp(),update_by=$4,view_type=COALESCE($7,view_type),comment_status=COALESCE($8,comment_status) WHERE id=$5 AND user_id=$6 AND deleted=0 RETURNING id,article_title,article_content,prospectus,update_time,view_type,comment_status",&[&request.article_title.trim(),&request.article_content,&request.prospectus,&editor,&id,&actor,&request.view_type,&request.comment_status]).await.map_err(internal)?;
	let mut article = EditableArticle::try_from(&row).map_err(internal)?;
	if let Some(names) = &request.tags {
		article.tags = crate::services::article_tags::resolve(tx, names)
			.await
			.map_err(internal)?;
		tx.execute(
			"DELETE FROM public.article_tags WHERE article_id=$1",
			&[&id],
		)
		.await
		.map_err(internal)?;
		for tag in &article.tags {
			tx.execute(
				"INSERT INTO public.article_tags(article_id,tag_id) VALUES($1,$2)",
				&[&id, &tag.id],
			)
			.await
			.map_err(internal)?;
		}
	} else {
		article.tags = crate::services::article_tags::for_article(tx, id)
			.await
			.map_err(internal)?;
	}
	Ok(article)
}
#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn rejects_empty_and_oversized_edits() {
		let mut request = EditRequest {
			view_type: None,
			comment_status: None,
			tags: None,
			article_title: "标题".into(),
			article_content: "# Markdown".into(),
			prospectus: None,
			expected_update_time: None,
		};
		assert!(validate(&request));
		request.article_title = " ".into();
		assert!(!validate(&request));
		request.article_title = "a".repeat(256);
		assert!(!validate(&request));
		request.article_title = "标题".into();
		request.article_content = " ".into();
		assert!(!validate(&request));
	}
	#[test]
	fn rejects_attempts_to_change_author() {
		assert!(
			serde_json::from_value::<EditRequest>(
				serde_json::json!({"article_title":"x","article_content":"x","user_id":99})
			)
			.is_err()
		);
	}
}

#[cfg(test)]
mod live_tests {
	use super::*;
	#[actix_web::test]
	#[ignore = "requires PostgreSQL tunnel; all fixture changes roll back"]
	async fn live_editor_permissions_conflicts_and_markdown_roundtrip() {
		dotenvy::dotenv().ok();
		let pool = crate::db::create_database_pool(
			std::env::var("DATABASE_HOST").unwrap(),
			std::env::var("DATABASE_PORT").unwrap(),
			std::env::var("DATABASE_NAME").unwrap(),
			std::env::var("DATABASE_USER").unwrap(),
			std::env::var("DATABASE_USER_PASSWORD").unwrap(),
		);
		assert_eq!(
			get_editable(None, web::Data::new(pool.clone()), web::Path::from(1))
				.await
				.status(),
			StatusCode::UNAUTHORIZED
		);
		let mut c = pool.get().await.unwrap();
		let tx = c.transaction().await.unwrap();
		let mut users = Vec::new();
		for n in 0..2 {
			let name = format!(
				"edt-{}-{n}",
				chrono::Utc::now().timestamp_nanos_opt().unwrap()
			);
			users.push(tx.query_one("INSERT INTO public.user(username,password) VALUES($1,'test-only') RETURNING id",&[&name]).await.unwrap().get::<_,i32>(0));
		}
		let row=tx.query_one("INSERT INTO public.article(user_id,sort_id,label_id,article_title,article_content) VALUES($1,0,0,'test','# original') RETURNING id,update_time",&[&users[0]]).await.unwrap();
		let id: i32 = row.get(0);
		let request = EditRequest {
			view_type: None,
			comment_status: None,
			tags: Some(vec!["JavaScript".into(), "JavaScrip".into()]),
			article_title: "Updated title".into(),
			article_content: "# Markdown\n\n| A | B |\n|---|---|\n| 1 | 2 |\n"
				.into(),
			prospectus: Some("summary".into()),
			expected_update_time: row.get(1),
		};
		assert_eq!(
			persist(&tx, id, users[1], &request)
				.await
				.err()
				.unwrap()
				.status(),
			StatusCode::FORBIDDEN
		);
		let updated = persist(&tx, id, users[0], &request).await.ok().unwrap();
		assert_eq!(updated.article_content, request.article_content);
		assert_eq!(updated.tags.len(), 1);
		assert_eq!(
			crate::services::article_tags::for_article(&*tx, id)
				.await
				.unwrap()
				.len(),
			1
		);
		assert_eq!(updated.article_title, request.article_title);
		assert_eq!(updated.view_type, "public");
		assert_eq!(updated.comment_status, 1);
		let settings = EditRequest {
			view_type: Some("authenticated".into()),
			comment_status: Some(0),
			tags: None,
			article_title: updated.article_title.clone(),
			article_content: updated.article_content.clone(),
			prospectus: updated.prospectus.clone(),
			expected_update_time: updated.update_time,
		};
		let configured = persist(&tx, id, users[0], &settings).await.ok().unwrap();
		assert_eq!(configured.view_type, "authenticated");
		assert_eq!(configured.comment_status, 0);

		assert_eq!(
			persist(&tx, id, users[0], &request)
				.await
				.err()
				.unwrap()
				.status(),
			StatusCode::CONFLICT
		);
		tx.execute(
			"UPDATE public.user SET user_status=0 WHERE id=$1",
			&[&users[0]],
		)
		.await
		.unwrap();
		assert_eq!(
			persist(&tx, id, users[0], &request)
				.await
				.err()
				.unwrap()
				.status(),
			StatusCode::FORBIDDEN
		);
		tx.execute(
			"UPDATE public.user SET user_status=1 WHERE id=$1",
			&[&users[0]],
		)
		.await
		.unwrap();
		tx.execute("UPDATE public.article SET deleted=1 WHERE id=$1", &[&id])
			.await
			.unwrap();
		assert_eq!(
			persist(&tx, id, users[0], &request)
				.await
				.err()
				.unwrap()
				.status(),
			StatusCode::NOT_FOUND
		);
		tx.rollback().await.unwrap();
		assert_eq!(
			c.query_one("SELECT count(*) FROM public.article WHERE id=$1", &[&id])
				.await
				.unwrap()
				.get::<_, i64>(0),
			0
		);
	}
}
