use crate::models::tag::Tag;
use tokio_postgres::{GenericClient, Transaction};
pub const TAG_LOCK: i64 = 73429105;
pub fn normalize(value: &str) -> String {
	value
		.chars()
		.filter(|c| !c.is_whitespace())
		.flat_map(char::to_lowercase)
		.collect()
}
pub fn similarity(a: &str, b: &str) -> f64 {
	let a: Vec<char> = normalize(a).chars().collect();
	let b: Vec<char> = normalize(b).chars().collect();
	if a == b {
		return 1.0;
	}
	if a.len().min(b.len()) <= 2 {
		return 0.0;
	}
	let mut row: Vec<usize> = (0..=b.len()).collect();
	for (i, x) in a.iter().enumerate() {
		let mut next = vec![i + 1];
		for (j, y) in b.iter().enumerate() {
			next.push(
				(row[j + 1] + 1)
					.min(next[j] + 1)
					.min(row[j] + usize::from(x != y)),
			);
		}
		row = next;
	}
	1.0 - row[b.len()] as f64 / a.len().max(b.len()) as f64
}
pub fn best_match<'a>(name: &str, tags: &'a [Tag]) -> Option<&'a Tag> {
	let exact = tags
		.iter()
		.find(|tag| normalize(name) == normalize(&tag.tag_name));
	exact.or_else(|| {
		tags.iter()
			.filter_map(|tag| {
				let score = similarity(name, &tag.tag_name);
				(score >= 0.8).then_some((tag, score))
			})
			.max_by(|a, b| a.1.total_cmp(&b.1).then_with(|| b.0.id.cmp(&a.0.id)))
			.map(|(tag, _)| tag)
	})
}
pub fn valid_names(names: &[String]) -> bool {
	names.len() <= 10
		&& names
			.iter()
			.all(|name| !normalize(name).is_empty() && name.chars().count() <= 40)
}
pub async fn list<C: GenericClient + Sync>(
	c: &C,
) -> Result<Vec<Tag>, tokio_postgres::Error> {
	c.query("SELECT id,tag_name,status FROM public.tag ORDER BY id", &[])
		.await?
		.iter()
		.map(Tag::try_from)
		.collect()
}
pub async fn for_article<C: GenericClient + Sync>(
	c: &C,
	id: i32,
) -> Result<Vec<Tag>, tokio_postgres::Error> {
	c.query("SELECT t.id,t.tag_name,t.status FROM public.tag t JOIN public.article_tags a ON a.tag_id=t.id WHERE a.article_id=$1 ORDER BY t.id",&[&id]).await?.iter().map(Tag::try_from).collect()
}
pub async fn resolve(
	tx: &Transaction<'_>,
	names: &[String],
) -> Result<Vec<Tag>, tokio_postgres::Error> {
	tx.query_one("SELECT pg_advisory_xact_lock($1)", &[&TAG_LOCK])
		.await?;
	let mut tags = list(tx).await?;
	let mut selected = Vec::new();
	for name in names {
		let tag = if let Some(tag) = best_match(name, &tags) {
			tag.clone()
		} else {
			let row=tx.query_one("INSERT INTO public.tag(tag_name,status) VALUES($1,0) RETURNING id,tag_name,status",&[&name.trim()]).await?;
			let tag = Tag::try_from(&row)?;
			tags.push(tag.clone());
			tag
		};
		if !selected.iter().any(|t: &Tag| t.id == tag.id) {
			selected.push(tag);
		}
	}
	Ok(selected)
}
#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn exact_case_spacing_typos_and_distinct_tags() {
		let tags = vec![
			Tag::new(1, "JavaScript".into(), 0),
			Tag::new(2, "Java".into(), 0),
			Tag::new(3, "C++".into(), 0),
		];
		assert_eq!(best_match(" javascript ", &tags).unwrap().id, 1);
		assert_eq!(best_match("Java Script", &tags).unwrap().id, 1);
		assert_eq!(best_match("JavaScrip", &tags).unwrap().id, 1);
		assert_eq!(best_match("Java", &tags).unwrap().id, 2);
		assert!(best_match("C#", &tags).is_none());
		assert!(best_match("Python", &tags).is_none());
	}
	#[test]
	fn limits_and_unicode() {
		assert!(!valid_names(&[" ".into()]));
		assert!(!valid_names(&vec!["tag".into(); 11]));
		assert_eq!(similarity("数据库优化", "数据库优话"), 0.8);
	}
}

#[cfg(test)]
mod live_tests {
	use super::*;
	#[actix_web::test]
	#[ignore = "requires PostgreSQL tunnel; all writes roll back"]
	async fn live_similar_names_reuse_one_tag() {
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
		let name = format!(
			"matching-{}",
			chrono::Utc::now().timestamp_nanos_opt().unwrap()
		);
		let a = resolve(&tx, &[name.clone()]).await.unwrap();
		let b = resolve(
			&tx,
			&[name.to_uppercase(), format!("{name}x"), format!(" {name} ")],
		)
		.await
		.unwrap();
		assert_eq!(a.len(), 1);
		assert_eq!(b.len(), 1);
		assert_eq!(a[0].id, b[0].id);
		let id = a[0].id;
		tx.rollback().await.unwrap();
		assert_eq!(
			c.query_one("SELECT count(*) FROM public.tag WHERE id=$1", &[&id])
				.await
				.unwrap()
				.get::<_, i64>(0),
			0
		);
	}
}
