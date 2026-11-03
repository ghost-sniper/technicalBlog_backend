use deadpool_postgres::Client;

pub struct StoredImage {
	pub content_type: String,
	pub content: Vec<u8>,
}

// Keep the public image ID stable when swapping the binary storage provider.
pub(crate) trait ArticleImageStorage {
	async fn put(
		&self,
		article_id: i32,
		author_id: i32,
		content_type: &str,
		content: &[u8],
	) -> anyhow::Result<Option<i64>>;

	async fn get(
		&self,
		id: i64,
		viewer_id: Option<i32>,
	) -> anyhow::Result<Option<StoredImage>>;
}

pub struct PostgresArticleImages<'a>(pub &'a Client);

impl ArticleImageStorage for PostgresArticleImages<'_> {
	async fn put(
		&self,
		article_id: i32,
		author_id: i32,
		content_type: &str,
		content: &[u8],
	) -> anyhow::Result<Option<i64>> {
		let row = self.0.query_opt(
			"INSERT INTO public.article_images (article_id, content_type, content)
			 SELECT a.id, $3, $4
			 FROM public.article a JOIN public.user u ON u.id = a.user_id
			 WHERE a.id = $1 AND a.user_id = $2 AND a.deleted = 0
			   AND u.user_status = 1 AND u.deleted = 0
			 RETURNING id",
			&[&article_id, &author_id, &content_type, &content],
		).await?;
		Ok(row.map(|row| row.get("id")))
	}

	async fn get(
		&self,
		id: i64,
		viewer_id: Option<i32>,
	) -> anyhow::Result<Option<StoredImage>> {
		let row = self
			.0
			.query_opt(
				"SELECT i.content_type, i.content FROM public.article_images i
			 JOIN public.article a ON a.id = i.article_id
			 WHERE i.id = $1 AND a.deleted = 0 AND (
			   strpos(a.article_content, '](/api/article-images/' || i.id::text || ')') > 0
			   OR EXISTS (
			     SELECT 1 FROM public.user u WHERE u.id = a.user_id AND u.id = $2
			       AND u.user_status = 1 AND u.deleted = 0
			   )
			 )",
				&[&id, &viewer_id],
			)
			.await?;
		Ok(row.map(|row| StoredImage {
			content_type: row.get("content_type"),
			content: row.get("content"),
		}))
	}
}
