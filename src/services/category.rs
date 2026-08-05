use crate::dao::category::{
	delete_category_by_id, insert_category, query_all_categories,
	query_category_by_id, update_category_by_id,
};
use crate::models::category::Category;
use crate::services::{CategoryError, CategoryResult};
use deadpool_postgres::Client;

pub async fn get_category_list(client: &Client) -> CategoryResult<Vec<Category>> {
	let result = query_all_categories(client).await;
	if result.is_empty() {
		Err(CategoryError::CategoryNotFound)
	} else {
		Ok(result)
	}
}

pub async fn get_category(client: &Client, id: i64) -> CategoryResult<Category> {
	let result = query_category_by_id(client, id).await;
	if result.is_none() {
		Err(CategoryError::CategoryNotFound)
	} else {
		Ok(result.unwrap())
	}
}

pub async fn delete_category(client: &Client, id: i64) -> CategoryResult<u64> {
	let result = delete_category_by_id(client, id).await;
	if result == 0 {
		Err(CategoryError::CategoryDeleteFailed)
	} else {
		Ok(result)
	}
}

pub async fn create_category(
	client: &Client,
	name: &str,
	description: &str,
	is_deleted: i16,
) -> CategoryResult<u64> {
	let effected_row = insert_category(client, name, description, is_deleted).await;
	if effected_row == 0 {
		Err(CategoryError::CategoryCreationFailed)
	} else {
		Ok(effected_row)
	}
}

pub async fn update_category(
	client: &Client,
	id: i64,
	name: &str,
	description: &str,
	is_deleted: i16,
) -> CategoryResult<u64> {
	let effected_row =
		update_category_by_id(client, id, name, description, is_deleted).await;
	if effected_row == 0 {
		Err(CategoryError::CategoryModificationFailed)
	} else {
		Ok(effected_row)
	}
}
