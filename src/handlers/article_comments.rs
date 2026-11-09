use super::article_access::{fail, internal, readable, viewer};
use actix_identity::Identity;
use actix_web::{HttpResponse, http::StatusCode, web};
use deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio_postgres::GenericClient;

const COMMENT_SELECT: &str = "SELECT to_jsonb(c) - 'deleted' || jsonb_build_object('username',u.username,'avatar',u.avatar) AS comment FROM public.article_comments c JOIN public.\"user\" u ON u.id=c.user_id";
#[derive(Deserialize)]
pub struct Page {
	pub page: Option<i64>,
	pub after: Option<i64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewComment {
	pub content: String,
	pub reply_to_comment_id: Option<i64>,
}
fn valid(request: &NewComment) -> bool {
	(1..=2000).contains(&request.content.trim().chars().count())
		&& request.reply_to_comment_id.is_none_or(|id| id > 0)
}
fn ok(value: Value) -> HttpResponse {
	HttpResponse::Ok()
		.insert_header(("Cache-Control", "private, no-store"))
		.json(value)
}
async fn active_user<C: GenericClient + Sync>(
	c: &C,
	uid: Option<i32>,
	lock: bool,
) -> Result<Option<(i32, bool)>, HttpResponse> {
	let sql = format!(
		"SELECT id,user_type FROM public.\"user\" WHERE id=$1 AND user_status=1 AND deleted=0{}",
		if lock { " FOR SHARE" } else { "" }
	);
	Ok(c.query_opt(&sql, &[&uid])
		.await
		.map_err(internal)?
		.map(|r| (r.get(0), r.get::<_, i16>(1) == 0)))
}
async fn comment<C: GenericClient + Sync>(
	c: &C,
	id: i64,
) -> Result<Value, HttpResponse> {
	Ok(
		c.query_one(&format!("{COMMENT_SELECT} WHERE c.id=$1"), &[&id])
			.await
			.map_err(internal)?
			.get(0),
	)
}
async fn reply_page<C: GenericClient + Sync>(
	c: &C,
	article: i32,
	root: i64,
	after: i64,
	limit: i64,
) -> Result<Vec<Value>, HttpResponse> {
	Ok(c.query(&format!("{COMMENT_SELECT} WHERE c.article_id=$1 AND c.root_id=$2 AND NOT c.deleted AND c.id>$3 ORDER BY c.id LIMIT $4"), &[&article,&root,&after,&limit]).await.map_err(internal)?.into_iter().map(|r| r.get(0)).collect())
}
pub async fn list(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	id: web::Path<i32>,
	query: web::Query<Page>,
) -> HttpResponse {
	let result = async {
        let page = query.page.unwrap_or(1);
        let offset = page.checked_sub(1).and_then(|v| v.checked_mul(10)).filter(|v| *v>=0).ok_or_else(|| fail(StatusCode::BAD_REQUEST,"分页参数无效"))?;
        let mut c = pool.get().await.map_err(internal)?;
        let tx = c.transaction().await.map_err(internal)?;
        let uid = viewer(identity);
        let user = active_user(&*tx, uid, false).await?;
        let article = readable(&*tx,*id,uid,false).await?;
        let can_delete = user.is_some_and(|(uid,admin)| admin || uid==article.get::<_,i32>("user_id"));
        let enabled = article.get::<_,i16>("comment_status")==1;
        let count: i64 = tx.query_one("SELECT count(*) FROM public.article_comments WHERE article_id=$1 AND root_id IS NULL AND NOT deleted", &[&*id]).await.map_err(internal)?.get(0);
        let rows = tx.query(&format!("{COMMENT_SELECT} WHERE c.article_id=$1 AND c.root_id IS NULL AND NOT c.deleted ORDER BY c.created_at DESC,c.id DESC LIMIT 10 OFFSET $2"), &[&*id,&offset]).await.map_err(internal)?;
        let mut comments = Vec::new();
        for row in rows {
            let mut item: Value = row.get(0);
            let root = item["id"].as_i64().unwrap();
            let replies = reply_page(&*tx,*id,root,0,3).await?;
            let total: i64 = tx.query_one("SELECT count(*) FROM public.article_comments WHERE root_id=$1 AND NOT deleted", &[&root]).await.map_err(internal)?.get(0);
            item["next_cursor"] = replies.last().map(|v| v["id"].clone()).unwrap_or(json!(0));
            item["replies"] = json!(replies);
            item["reply_count"] = json!(total);
            comments.push(item);
        }
        Ok(ok(json!({"comments":comments,"count":count,"page":page,"can_comment":enabled && user.is_some(),"can_delete":can_delete,"comments_enabled":enabled,"logged_in":user.is_some()})))
    }.await;
	result.unwrap_or_else(|e: HttpResponse| e)
}
pub async fn replies(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	path: web::Path<(i32, i64)>,
	query: web::Query<Page>,
) -> HttpResponse {
	let result = async {
        let (article,root)=path.into_inner();
        let after=query.after.unwrap_or(0);
        if after<0 { return Err(fail(StatusCode::BAD_REQUEST,"回复游标无效")); }
        let mut c=pool.get().await.map_err(internal)?;
        let tx=c.transaction().await.map_err(internal)?;
        readable(&*tx,article,viewer(identity),false).await?;
        let exists=tx.query_opt("SELECT id FROM public.article_comments WHERE id=$1 AND article_id=$2 AND root_id IS NULL AND NOT deleted", &[&root,&article]).await.map_err(internal)?;
        if exists.is_none() { return Err(fail(StatusCode::NOT_FOUND,"评论已删除")); }
        let mut items=reply_page(&*tx,article,root,after,11).await?;
        let has_more=items.len()>10;
        items.truncate(10);
        let cursor=items.last().map(|v| v["id"].clone()).unwrap_or(json!(after));
        Ok(ok(json!({"replies":items,"next_cursor":cursor,"has_more":has_more})))
    }.await;
	result.unwrap_or_else(|e: HttpResponse| e)
}
async fn write_context(
	tx: &tokio_postgres::Transaction<'_>,
	article: i32,
	uid: Option<i32>,
) -> Result<(i32, bool, i16), HttpResponse> {
	let uid = uid.ok_or_else(|| fail(StatusCode::UNAUTHORIZED, "请先登录"))?;
	let (_, admin) = active_user(tx, Some(uid), true)
		.await?
		.ok_or_else(|| fail(StatusCode::FORBIDDEN, "账号已被冻结或删除"))?;
	let row = readable(tx, article, Some(uid), true).await?;
	Ok((
		uid,
		admin || row.get::<_, i32>("user_id") == uid,
		row.get("comment_status"),
	))
}
async fn insert(
	tx: &tokio_postgres::Transaction<'_>,
	article: i32,
	uid: Option<i32>,
	request: &NewComment,
) -> Result<Value, HttpResponse> {
	if !valid(request) {
		return Err(fail(
			StatusCode::BAD_REQUEST,
			"评论须为 1～2000 字，回复目标须有效",
		));
	}
	let (uid, _, enabled) = write_context(tx, article, uid).await?;
	if enabled != 1 {
		return Err(fail(StatusCode::FORBIDDEN, "该文章已关闭评论"));
	}
	let mut root: Option<i64> = None;
	let mut target_user: Option<i32> = None;
	let mut target_name: Option<String> = None;
	if let Some(target) = request.reply_to_comment_id {
		let row=tx.query_opt("SELECT c.id,c.root_id,c.user_id,u.username FROM public.article_comments c JOIN public.\"user\" u ON u.id=c.user_id WHERE c.id=$1 AND c.article_id=$2 AND NOT c.deleted", &[&target,&article]).await.map_err(internal)?.ok_or_else(||fail(StatusCode::NOT_FOUND,"回复目标不存在或已删除"))?;
		root = Some(row.get::<_, Option<i64>>("root_id").unwrap_or(target));
		let exists=tx.query_opt("SELECT id FROM public.article_comments WHERE id=$1 AND article_id=$2 AND root_id IS NULL AND NOT deleted", &[&root,&article]).await.map_err(internal)?;
		if exists.is_none() {
			return Err(fail(StatusCode::NOT_FOUND, "一级评论已删除"));
		}
		target_user = Some(row.get("user_id"));
		target_name = Some(row.get("username"));
	}
	let id:i64=tx.query_one("INSERT INTO public.article_comments(article_id,user_id,root_id,reply_to_comment_id,reply_to_user_id,reply_to_username,content) VALUES($1,$2,$3,$4,$5,$6,$7) RETURNING id", &[&article,&uid,&root,&request.reply_to_comment_id,&target_user,&target_name,&request.content.trim()]).await.map_err(internal)?.get(0);
	comment(tx, id).await
}
pub async fn create(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	id: web::Path<i32>,
	request: web::Json<NewComment>,
) -> HttpResponse {
	let result = async {
		let mut c = pool.get().await.map_err(internal)?;
		let tx = c.transaction().await.map_err(internal)?;
		let item = insert(&*tx, *id, viewer(identity), &request).await?;
		tx.commit().await.map_err(internal)?;
		Ok(HttpResponse::Created().json(item))
	}
	.await;
	result.unwrap_or_else(|e: HttpResponse| e)
}
async fn remove(
	tx: &tokio_postgres::Transaction<'_>,
	article: i32,
	id: i64,
	uid: Option<i32>,
) -> Result<(), HttpResponse> {
	let (_, can_delete, _) = write_context(tx, article, uid).await?;
	if !can_delete {
		return Err(fail(
			StatusCode::FORBIDDEN,
			"仅文章所有者或管理员可以删除评论",
		));
	}
	let row=tx.query_opt("SELECT root_id,deleted FROM public.article_comments WHERE id=$1 AND article_id=$2", &[&id,&article]).await.map_err(internal)?.ok_or_else(||fail(StatusCode::NOT_FOUND,"评论不存在"))?;
	if row.get::<_, bool>("deleted") {
		return Ok(());
	}
	if row.get::<_, Option<i64>>("root_id").is_none() {
		tx.execute("UPDATE public.article_comments SET deleted=true WHERE article_id=$1 AND (id=$2 OR root_id=$2)", &[&article,&id]).await.map_err(internal)?;
	} else {
		tx.execute(
			"UPDATE public.article_comments SET deleted=true WHERE id=$1",
			&[&id],
		)
		.await
		.map_err(internal)?;
	}
	Ok(())
}
pub async fn delete(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	path: web::Path<(i32, i64)>,
) -> HttpResponse {
	let result = async {
		let (article, id) = path.into_inner();
		let mut c = pool.get().await.map_err(internal)?;
		let tx = c.transaction().await.map_err(internal)?;
		remove(&*tx, article, id, viewer(identity)).await?;
		tx.commit().await.map_err(internal)?;
		Ok(HttpResponse::NoContent().finish())
	}
	.await;
	result.unwrap_or_else(|e: HttpResponse| e)
}
#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn validates_plain_text_and_identity() {
		for (text, expected) in [
			(" ".to_string(), false),
			("中".repeat(2000), true),
			("中".repeat(2001), false),
			("\n hello \n".to_string(), true),
		] {
			assert_eq!(
				valid(&NewComment {
					content: text,
					reply_to_comment_id: None
				}),
				expected
			);
		}
		assert!(
			serde_json::from_value::<NewComment>(
				json!({"content":"hi","user_id":4})
			)
			.is_err()
		);
		assert!(!valid(&NewComment {
			content: "hi".into(),
			reply_to_comment_id: Some(-1)
		}));
	}
}

#[cfg(test)]
mod live_tests {
	use super::*;
	#[actix_web::test]
	#[ignore = "requires configured PostgreSQL; migration and fixtures roll back"]
	async fn live_comment_permissions_threads_and_deletion() {
		dotenvy::dotenv().ok();
		let pool = crate::db::pool_from_env().unwrap();
		let mut client = pool.get().await.unwrap();
		let tx = client.transaction().await.unwrap();
		let exists: bool = tx
			.query_one(
				"SELECT to_regclass('public.article_comments') IS NOT NULL",
				&[],
			)
			.await
			.unwrap()
			.get(0);
		if !exists {
			tx.batch_execute(
				&include_str!("../../migrations/20261109_article_comments.sql")
					.replace("BEGIN;", "")
					.replace("COMMIT;", ""),
			)
			.await
			.unwrap();
		}
		let stamp = chrono::Utc::now().timestamp_nanos_opt().unwrap();
		let mut users = Vec::new();
		for (n, role, status) in [(0, 1_i16, 1_i16), (1, 1, 1), (2, 0, 1), (3, 0, 0)]
		{
			let name = format!("cmt-test-{stamp}-{n}");
			users.push(tx.query_one("INSERT INTO public.\"user\"(username,password,user_type,user_status,deleted) VALUES($1,'test-only',$2,$3,0) RETURNING id", &[&name,&role,&status]).await.unwrap().get::<_,i32>(0));
		}
		let owner = users[0];
		let reader = users[1];
		let admin = users[2];
		let frozen = users[3];
		let article:i32=tx.query_one("INSERT INTO public.article(user_id,sort_id,label_id,article_title,article_content,view_type,comment_status,deleted) VALUES($1,0,0,'comment fixture','test','public',1,0) RETURNING id", &[&owner]).await.unwrap().get(0);
		for mode in ["public", "authenticated", "private", "legacy"] {
			tx.execute(
				"UPDATE public.article SET view_type=$1 WHERE id=$2",
				&[&mode, &article],
			)
			.await
			.unwrap();
			for uid in [None, Some(owner), Some(reader), Some(admin), Some(frozen)] {
				let expected = mode == "public"
					|| uid == Some(owner)
					|| uid == Some(admin)
					|| (mode == "authenticated" && uid == Some(reader));
				assert_eq!(
					readable(&*tx, article, uid, false).await.is_ok(),
					expected,
					"mode={mode} uid={uid:?}"
				);
			}
		}
		tx.execute(
			"UPDATE public.article SET view_type='public' WHERE id=$1",
			&[&article],
		)
		.await
		.unwrap();
		let request = NewComment {
			content: "  root\nplain <b>text</b>  ".into(),
			reply_to_comment_id: None,
		};
		assert_eq!(
			insert(&*tx, article, None, &request)
				.await
				.unwrap_err()
				.status(),
			StatusCode::UNAUTHORIZED
		);
		assert_eq!(
			insert(&*tx, article, Some(frozen), &request)
				.await
				.unwrap_err()
				.status(),
			StatusCode::FORBIDDEN
		);
		let root = insert(&*tx, article, Some(reader), &request).await.unwrap();
		assert_eq!(root["content"], "root\nplain <b>text</b>");
		let root_id = root["id"].as_i64().unwrap();
		let reply = insert(
			&*tx,
			article,
			Some(owner),
			&NewComment {
				content: "first reply".into(),
				reply_to_comment_id: Some(root_id),
			},
		)
		.await
		.unwrap();
		let reply_id = reply["id"].as_i64().unwrap();
		let nested = insert(
			&*tx,
			article,
			Some(reader),
			&NewComment {
				content: "reply to reply".into(),
				reply_to_comment_id: Some(reply_id),
			},
		)
		.await
		.unwrap();
		assert_eq!(nested["root_id"], root_id);
		assert_eq!(nested["reply_to_user_id"], owner);
		assert_eq!(
			remove(&*tx, article, root_id, Some(reader))
				.await
				.unwrap_err()
				.status(),
			StatusCode::FORBIDDEN
		);
		for n in 0..14 {
			insert(
				&*tx,
				article,
				Some(reader),
				&NewComment {
					content: format!("reply {n}"),
					reply_to_comment_id: Some(root_id),
				},
			)
			.await
			.unwrap();
		}
		let preview = reply_page(&*tx, article, root_id, 0, 3).await.unwrap();
		assert_eq!(preview.len(), 3);
		let more = reply_page(
			&*tx,
			article,
			root_id,
			preview.last().unwrap()["id"].as_i64().unwrap(),
			10,
		)
		.await
		.unwrap();
		assert_eq!(more.len(), 10);
		assert!(
			more[0]["id"].as_i64().unwrap() > preview[2]["id"].as_i64().unwrap()
		);
		let other:i32=tx.query_one("INSERT INTO public.article(user_id,sort_id,label_id,article_title,article_content,view_type,comment_status,deleted) VALUES($1,0,0,'other','test','public',1,0) RETURNING id", &[&owner]).await.unwrap().get(0);
		assert_eq!(
			insert(
				&*tx,
				other,
				Some(reader),
				&NewComment {
					content: "cross article".into(),
					reply_to_comment_id: Some(root_id)
				}
			)
			.await
			.unwrap_err()
			.status(),
			StatusCode::NOT_FOUND
		);
		remove(&*tx, article, reply_id, Some(admin)).await.unwrap();
		assert_eq!(
			comment(&*tx, nested["id"].as_i64().unwrap()).await.unwrap()["reply_to_username"],
			reply["username"]
		);
		assert_eq!(
			insert(
				&*tx,
				article,
				Some(reader),
				&NewComment {
					content: "deleted target".into(),
					reply_to_comment_id: Some(reply_id)
				}
			)
			.await
			.unwrap_err()
			.status(),
			StatusCode::NOT_FOUND
		);
		tx.execute(
			"UPDATE public.article SET comment_status=0 WHERE id=$1",
			&[&article],
		)
		.await
		.unwrap();
		assert_eq!(
			insert(&*tx, article, Some(owner), &request)
				.await
				.unwrap_err()
				.status(),
			StatusCode::FORBIDDEN
		);
		remove(&*tx, article, root_id, Some(owner)).await.unwrap();
		remove(&*tx, article, root_id, Some(owner)).await.unwrap();
		assert!(
			reply_page(&*tx, article, root_id, 0, 100)
				.await
				.unwrap()
				.is_empty()
		);
		let remaining:i64=tx.query_one("SELECT count(*) FROM public.article_comments WHERE article_id=$1 AND NOT deleted", &[&article]).await.unwrap().get(0);
		assert_eq!(remaining, 0);
		tx.rollback().await.unwrap();
	}
	#[actix_web::test]
	#[ignore = "requires BLOG_ISOLATED_TEST_DATABASE=1; uses disposable database only"]
	async fn live_comment_endpoints_search_images_and_concurrency() {
		assert_eq!(
			std::env::var("BLOG_ISOLATED_TEST_DATABASE").as_deref(),
			Ok("1"),
			"Use a disposable database only"
		);
		let pool = crate::db::pool_from_env().unwrap();
		let mut c = pool.get().await.unwrap();
		c.batch_execute(include_str!(
			"../../migrations/20261109_article_comments.sql"
		))
		.await
		.unwrap();
		let tx = c.transaction().await.unwrap();
		let owner:i32=tx.query_one("INSERT INTO public.\"user\"(username,password,user_type) VALUES('isolated-owner','test',1) RETURNING id", &[]).await.unwrap().get(0);
		let reader:i32=tx.query_one("INSERT INTO public.\"user\"(username,password,user_type) VALUES('isolated-reader','test',1) RETURNING id", &[]).await.unwrap().get(0);
		let article:i32=tx.query_one("INSERT INTO public.article(user_id,sort_id,label_id,article_title,article_content,view_type,comment_status) VALUES($1,0,0,'isolated-marker','isolated-marker','public',1) RETURNING id", &[&owner]).await.unwrap().get(0);
		let mut root_id = 0;
		for n in 0..11 {
			root_id = insert(
				&*tx,
				article,
				Some(reader),
				&NewComment {
					content: format!("root {n}"),
					reply_to_comment_id: None,
				},
			)
			.await
			.unwrap()["id"]
				.as_i64()
				.unwrap();
		}
		for n in 0..15 {
			insert(
				&*tx,
				article,
				Some(reader),
				&NewComment {
					content: format!("reply {n}"),
					reply_to_comment_id: Some(root_id),
				},
			)
			.await
			.unwrap();
		}
		tx.commit().await.unwrap();
		let response = list(
			None,
			web::Data::new(pool.clone()),
			web::Path::from(article),
			web::Query(Page {
				page: Some(1),
				after: None,
			}),
		)
		.await;
		assert_eq!(response.status(), StatusCode::OK);
		let body: Value = serde_json::from_slice(
			&actix_web::body::to_bytes(response.into_body())
				.await
				.unwrap(),
		)
		.unwrap();
		assert_eq!(body["count"], 11);
		assert_eq!(body["comments"].as_array().unwrap().len(), 10);
		assert_eq!(body["comments"][0]["replies"].as_array().unwrap().len(), 3);
		assert_eq!(body["can_comment"], false);
		let after = body["comments"][0]["next_cursor"].as_i64().unwrap();
		let response = replies(
			None,
			web::Data::new(pool.clone()),
			web::Path::from((article, root_id)),
			web::Query(Page {
				page: None,
				after: Some(after),
			}),
		)
		.await;
		let body: Value = serde_json::from_slice(
			&actix_web::body::to_bytes(response.into_body())
				.await
				.unwrap(),
		)
		.unwrap();
		assert_eq!(body["replies"].as_array().unwrap().len(), 10);
		assert_eq!(body["has_more"], true);
		let article_response = super::super::article::get_article(
			None,
			web::Data::new(pool.clone()),
			web::Path::from(article),
		)
		.await;
		let body: Value = serde_json::from_slice(
			&actix_web::body::to_bytes(article_response.into_body())
				.await
				.unwrap(),
		)
		.unwrap();
		assert!(body["message"].get("view_value").is_none());
		use crate::services::article_images::{
			ArticleImageStorage, PostgresArticleImages,
		};
		let image = PostgresArticleImages(&c)
			.put(article, owner, "image/png", &[1, 2, 3])
			.await
			.unwrap()
			.unwrap();
		assert!(
			PostgresArticleImages(&c)
				.get(image, None)
				.await
				.unwrap()
				.is_none()
		);
		c.execute(
			"UPDATE public.article SET article_content=$1 WHERE id=$2",
			&[&format!("![image](/api/article-images/{image})"), &article],
		)
		.await
		.unwrap();
		assert!(
			PostgresArticleImages(&c)
				.get(image, None)
				.await
				.unwrap()
				.is_some()
		);
		c.execute(
			"UPDATE public.article SET view_type='private' WHERE id=$1",
			&[&article],
		)
		.await
		.unwrap();
		assert!(
			PostgresArticleImages(&c)
				.get(image, None)
				.await
				.unwrap()
				.is_none()
		);
		assert!(
			PostgresArticleImages(&c)
				.get(image, Some(owner))
				.await
				.unwrap()
				.is_some()
		);
		for (uid, expected) in [(None, 0), (Some(reader), 0), (Some(owner), 1)] {
			let (count, items) = crate::dao::article::get_article_page(
				&c,
				1,
				10,
				"isolated-marker",
				None,
				uid,
			)
			.await
			.unwrap();
			assert_eq!(count, expected);
			assert_eq!(items.len(), expected as usize);
		}
		assert_eq!(
			list(
				None,
				web::Data::new(pool.clone()),
				web::Path::from(article),
				web::Query(Page {
					page: None,
					after: None
				})
			)
			.await
			.status(),
			StatusCode::NOT_FOUND
		);
		c.execute(
			"UPDATE public.article SET view_type='public' WHERE id=$1",
			&[&article],
		)
		.await
		.unwrap();
		let tx = c.transaction().await.unwrap();
		remove(&*tx, article, root_id, Some(owner)).await.unwrap();
		let mut other = pool.get().await.unwrap();
		let competing = other.transaction().await.unwrap();
		competing
			.batch_execute("SET LOCAL lock_timeout='100ms'")
			.await
			.unwrap();
		let request = NewComment {
			content: "concurrent reply".into(),
			reply_to_comment_id: Some(root_id),
		};
		assert_eq!(
			insert(&*competing, article, Some(reader), &request)
				.await
				.unwrap_err()
				.status(),
			StatusCode::INTERNAL_SERVER_ERROR
		);
		competing.rollback().await.unwrap();
		tx.commit().await.unwrap();
		let tx = other.transaction().await.unwrap();
		assert_eq!(
			insert(&*tx, article, Some(reader), &request)
				.await
				.unwrap_err()
				.status(),
			StatusCode::NOT_FOUND
		);
		tx.rollback().await.unwrap();
	}
}
