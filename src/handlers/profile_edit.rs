use super::article_access::{fail, internal};
use super::user::ProfileUser;
use actix_identity::Identity;
use actix_web::{HttpResponse, http::StatusCode, web};
use deadpool_postgres::Pool;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileEdit {
	username: String,
	email: Option<String>,
	phone_number: Option<String>,
	introduction: Option<String>,
}
fn clean(value: &Option<String>) -> Option<String> {
	value
		.as_ref()
		.map(|s| s.trim().to_owned())
		.filter(|s| !s.is_empty())
}
fn validate(r: &ProfileEdit) -> bool {
	let name = r.username.trim();
	!name.is_empty()
		&& name.chars().count() <= 32
		&& !name.chars().any(char::is_control)
		&& clean(&r.email).is_none_or(|s| {
			s.len() <= 32
				&& !s.chars().any(char::is_whitespace)
				&& s.split('@').count() == 2
				&& s.split('@').all(|v| !v.is_empty())
				&& s.rsplit('@').next().is_some_and(|v| {
					v.contains('.') && !v.starts_with('.') && !v.ends_with('.')
				})
		})
		&& clean(&r.phone_number).is_none_or(|s| {
			s.len() <= 16
				&& (5..=15).contains(&s.chars().filter(char::is_ascii_digit).count())
				&& s.chars()
					.enumerate()
					.all(|(i, c)| c.is_ascii_digit() || (i == 0 && c == '+'))
		})
		&& clean(&r.introduction).is_none_or(|s| s.chars().count() <= 4096)
}
async fn persist(
	tx: &tokio_postgres::Transaction<'_>,
	id: i32,
	r: &ProfileEdit,
) -> Result<ProfileUser, HttpResponse> {
	if !validate(r) {
		return Err(fail(
			StatusCode::BAD_REQUEST,
			"请检查资料：用户名 1～32 字，邮箱最多 32 字，手机号为 5～15 位数字（可带 +），简介最多 4096 字",
		));
	}
	// Serialize duplicate contact checks across profile updates.
	tx.query_one("SELECT pg_advisory_xact_lock(73429106)", &[])
		.await
		.map_err(internal)?;
	let current=tx.query_opt("SELECT email FROM public.\"user\" WHERE id=$1 AND user_status=1 AND deleted=0 FOR UPDATE", &[&id]).await.map_err(internal)?.ok_or_else(||fail(StatusCode::FORBIDDEN,"账号已被冻结或删除"))?;
	let email = clean(&r.email);
	let phone = clean(&r.phone_number);
	let intro = clean(&r.introduction);
	if current
		.get::<_, Option<String>>(0)
		.is_some_and(|v| !v.is_empty())
		&& email.is_none()
	{
		return Err(fail(StatusCode::BAD_REQUEST, "邮箱用于登录，不能清空"));
	}
	let duplicate=tx.query_opt("SELECT id FROM public.\"user\" WHERE id<>$1 AND (username=$2 OR (deleted=0 AND (($3::text IS NOT NULL AND lower(trim(email))=lower($3)) OR ($4::text IS NOT NULL AND phone_number=$4)))) LIMIT 1", &[&id,&r.username.trim(),&email,&phone]).await.map_err(internal)?;
	if duplicate.is_some() {
		return Err(fail(
			StatusCode::CONFLICT,
			"用户名、邮箱或手机号已被其他账号使用",
		));
	}
	let row=tx.query_one("UPDATE public.\"user\" SET username=$1,email=$2,phone_number=$3,introduction=$4,update_time=clock_timestamp(),update_by=$5 WHERE id=$6 RETURNING id,username,email,phone_number,avatar,introduction,user_lv AS user_level,create_time,user_status,deleted,user_type", &[&r.username.trim(),&email,&phone,&intro,&id.to_string(),&id]).await.map_err(|e| if e.code()==Some(&tokio_postgres::error::SqlState::UNIQUE_VIOLATION) {fail(StatusCode::CONFLICT,"用户名已被使用")} else {internal(e)})?;
	ProfileUser::try_from(&row).map_err(internal)
}
pub async fn save(
	identity: Option<Identity>,
	pool: web::Data<Pool>,
	request: web::Json<ProfileEdit>,
) -> HttpResponse {
	let (mut c, id) = match super::article_edit::actor(identity, &pool).await {
		Ok(v) => v,
		Err(e) => return e,
	};
	let result = async {
		let tx = c.transaction().await.map_err(internal)?;
		let user = persist(&*tx, id, &request).await?;
		tx.commit().await.map_err(internal)?;
		Ok(HttpResponse::Ok()
			.insert_header(("Cache-Control", "private, no-store"))
			.json(serde_json::json!({"user":user})))
	}
	.await;
	result.unwrap_or_else(|e: HttpResponse| e)
}
#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn validation_and_mass_assignment() {
		let mut r = ProfileEdit {
			username: " 用户 ".into(),
			email: Some("me@example.com".into()),
			phone_number: None,
			introduction: None,
		};
		assert!(validate(&r));
		r.username = " ".into();
		assert!(!validate(&r));
		r.username = "中".repeat(33);
		assert!(!validate(&r));
		r.username = "用户".into();
		r.email = Some("invalid".into());
		assert!(!validate(&r));
		for key in ["id", "user_type", "user_level", "password", "deleted"] {
			let mut v = serde_json::json!({"username":"user"});
			v[key] = serde_json::json!(1);
			assert!(serde_json::from_value::<ProfileEdit>(v).is_err());
		}
	}
	#[actix_web::test]
	#[ignore = "requires PostgreSQL; all fixture rows roll back"]
	async fn live_profile_edit() {
		dotenvy::dotenv().ok();
		let pool = crate::db::pool_from_env().unwrap();
		let mut c = pool.get().await.unwrap();
		let tx = c.transaction().await.unwrap();
		let name =
			format!("pf-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
		let id:i32=tx.query_one("INSERT INTO public.\"user\"(username,password,user_type,user_status,deleted,email) VALUES($1,'test',1,1,0,'profile-test@example.test') RETURNING id", &[&name]).await.unwrap().get(0);
		let r = ProfileEdit {
			username: format!("{name}x"),
			email: Some("profile-new@example.test".into()),
			phone_number: None,
			introduction: Some("  新简介  ".into()),
		};
		let saved = persist(&*tx, id, &r).await.ok().unwrap();
		assert_eq!(saved.introduction.as_deref(), Some("新简介"));
		assert_eq!(saved.user_type, 1);
		let other:i32=tx.query_one("INSERT INTO public.\"user\"(username,password,user_status) VALUES($1,'test',1) RETURNING id", &[&format!("{name}y")]).await.unwrap().get(0);
		assert_eq!(
			persist(&*tx, other, &r).await.err().unwrap().status(),
			StatusCode::CONFLICT
		);
		tx.execute(
			"UPDATE public.\"user\" SET user_status=0 WHERE id=$1",
			&[&id],
		)
		.await
		.unwrap();
		assert_eq!(
			persist(&*tx, id, &r).await.err().unwrap().status(),
			StatusCode::FORBIDDEN
		);
		tx.rollback().await.unwrap();
		assert_eq!(
			save(None, web::Data::new(pool), web::Json(r))
				.await
				.status(),
			StatusCode::UNAUTHORIZED
		);
	}
}
