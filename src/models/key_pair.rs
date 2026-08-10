use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use blog::TryFromRow;

#[derive(Debug, Clone, Serialize, Deserialize,TryFromRow)]
pub struct RsaKeyPair {
	pub public_key: String,
	pub private_key: String,
}
