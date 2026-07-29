use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsaKeyPair {
	pub public_key: String,
	pub private_key: String,
}

impl TryFrom<&Row> for RsaKeyPair {
	type Error = tokio_postgres::Error;

	fn try_from(value: &Row) -> Result<Self, Self::Error> {
		Ok(RsaKeyPair {
			public_key: value.try_get("public_key")?,
			private_key: value.try_get("private_key")?,
		})
	}
}
