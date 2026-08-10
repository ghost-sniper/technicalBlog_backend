use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub mod article;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct UnifiedResponse<T>
where
	T: Serialize + DeserializeOwned,
{
	pub status: String,
	pub code: i32,
	pub message: Option<T>,
}