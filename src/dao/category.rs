use crate::models::category::Category;
use deadpool_postgres::Client;

pub async fn query_all_categories(client: &Client) -> Vec<Category> {
	let rows = client.query("SELECT id, name,description,is_deleted, create_at, update_at  FROM poetize.category", &[]).await.unwrap();
	rows.iter()
		.map(|row| Category::try_from(row).unwrap())
		.collect()
}

pub async fn query_category_by_id(client: &Client, id: i64) -> Option<Category> {
	let row = client.query_one("SELECT id, name,description,is_deleted, create_at, update_at FROM poetize.category WHERE id = $1", &[&id]).await.unwrap();
	let category = Category::try_from(&row);
	if category.is_err() {
		None
	} else {
		Some(category.unwrap())
	}
}

pub async fn insert_category(
	client: &Client,
	name: &str,
	description: &str,
	is_deleted: i16,
) -> u64 {
	client
		.execute(
			"Insert into poetize.category(name, description,is_deleted) values ($1,$2,$3)",
			&[&name, &description, &is_deleted],
		)
		.await
		.expect("新增分类SQL执行失败")
}

pub async fn delete_category_by_id(client: &Client, id: i64) -> u64 {
	client
		.execute("Delete from poetize.category Where id = $1", &[&id])
		.await
		.expect("删除分类SQL执行失败")
}

pub async fn update_category_by_id(
	client: &Client,
	id: i64,
	name: &str,
	description: &str,
	is_deleted: i16,
) -> u64 {
	client
		.execute(
			"UPDATE poetize.category SET name = $1 , description = $2, is_deleted = $3, update_at = now() WHERE id = $4",
			&[&name, &description, &is_deleted, &id],
		)
		.await
		.expect("更新分类SQL执行失败")
}
