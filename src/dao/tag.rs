use crate::models::tag::Tag;
use deadpool_postgres::Client;

pub async fn query_all_tags(client: &Client) -> Vec<Tag> {
	let rows = client
		.query("SELECT * FROM poetize.tag", &[])
		.await
		.unwrap();
	rows.iter().map(|row| Tag::try_from(row).unwrap()).collect()
}

pub async fn insert_tag(client: &Client, tag_name: &str, status: i32) -> u64 {
	client
		.execute(
			"Insert into poetize.tag(tag_name,status) Values($1,$2)",
			&[&tag_name, &status],
		)
		.await
		.expect("新增标签SQL执行失败")
}

pub async fn delete_tag_by_id(client: &Client, id: i64) -> u64 {
	client
		.execute("Delete from poetize.tag Where id = $1", &[&id])
		.await
		.expect("删除标签SQL执行失败")
}

pub async fn update_tag_by_id(
	client: &Client,
	id: i64,
	tag_name: &str,
	status: i32,
) -> u64 {
	client
		.execute(
			"UPDATE poetize.tag SET tag_name = $1 , status = $2 WHERE id = $3",
			&[&tag_name, &status, &id],
		)
		.await
		.expect("更新标签SQL执行失败")
}

pub async fn query_tag_by_id(client: &Client, id: i64) -> Option<Tag> {
	let row = client
		.query_one("SELECT id, tag_name, status FROM poetize.tag Where id = $1", &[&id])
		.await
		.expect("通过ID查询标签SQL执行失败");
	let tag = Tag::try_from(&row);
	if tag.is_err() { None } else { tag.ok() }
}
