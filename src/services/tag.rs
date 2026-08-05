use crate::dao::tag::{
	delete_tag_by_id, insert_tag, query_all_tags, query_tag_by_id, update_tag_by_id,
};
use crate::models::tag::Tag;
use crate::services::{TagError, TagResult};
use deadpool_postgres::Client;

pub async fn get_tag_list(client: &Client) -> TagResult<Vec<Tag>> {
	let result = query_all_tags(client).await;
	if result.is_empty() {
		Err(TagError::TagNotFound)
	} else {
		Ok(result)
	}
}

pub async fn create_tag(
	client: &Client,
	tag_name: &str,
	status: i32,
) -> TagResult<u64> {
	let effected_row = insert_tag(client, tag_name, status).await;
	if effected_row == 0 {
		Err(TagError::TagCreationFailed)
	} else {
		Ok(effected_row)
	}
}

pub async fn delete_tag(client: &Client, id: i64) -> TagResult<u64> {
	let effected_row = delete_tag_by_id(client, id).await;
	if effected_row == 0 {
		Err(TagError::TagDeleteFailed)
	} else {
		Ok(effected_row)
	}
}

pub async fn update_tag(
	client: &Client,
	id: i64,
	new_tag_name: &str,
	new_status: i32,
) -> TagResult<u64> {
	let effected_row = update_tag_by_id(client, id, new_tag_name, new_status).await;
	if effected_row == 0 {
		Err(TagError::TagModificationFailed)
	} else {
		Ok(effected_row)
	}
}

pub async fn get_tag(client: &Client, id: i64) -> TagResult<Tag> {
	let result = query_tag_by_id(client, id).await;
	if result.is_none() {
		Err(TagError::TagNotFound)
	} else {
		Ok(result.unwrap())
	}
}
