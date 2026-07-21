use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tag {
	pub id: i64,
	pub tag_name: String,
	pub status: i32,
}
impl Tag {
	pub fn new(id: i64, tag_name: String, status: i32) -> Self {
		Self {
			id,
			tag_name,
			status,
		}
	}
}
