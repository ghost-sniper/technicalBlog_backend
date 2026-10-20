use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
use aws_lc_rs::rsa::{
	OAEP_SHA256_MGF1SHA256, OaepPrivateDecryptingKey, PrivateDecryptingKey,
};
use base64::{Engine as _, engine::general_purpose};
use cbc::Decryptor;
use pkcs8::SecretDocument;

type Aes256CbcDec = Decryptor<aes::Aes256>;

pub async fn decrypt_e2ee(
	aes_key_b64: &str,
	cipher_text_b64: &str,
	iv_b64: &str,
	private_key_pem: &str,
) -> anyhow::Result<String> {
	// Do not expose which stage of the encrypted login payload failed.
	decrypt_payload(aes_key_b64, cipher_text_b64, iv_b64, private_key_pem)
		.map_err(|_| anyhow::anyhow!("解密失败"))
}

fn decrypt_payload(
	aes_key_b64: &str,
	cipher_text_b64: &str,
	iv_b64: &str,
	private_key_pem: &str,
) -> anyhow::Result<String> {
	let mut cipher_text = general_purpose::STANDARD.decode(cipher_text_b64)?;
	let iv = general_purpose::STANDARD.decode(iv_b64)?;
	let aes_key = general_purpose::STANDARD.decode(aes_key_b64)?;
	anyhow::ensure!(
		iv.len() == 16 && !cipher_text.is_empty() && cipher_text.len() % 16 == 0
	);
	let (label, private_key_der) = SecretDocument::from_pem(private_key_pem)
		.map_err(|_| anyhow::anyhow!("解密失败"))?;
	anyhow::ensure!(label == "PRIVATE KEY");
	let private_key = PrivateDecryptingKey::from_pkcs8(private_key_der.as_bytes())
		.map_err(|_| anyhow::anyhow!("解密失败"))?;
	let private_key = OaepPrivateDecryptingKey::new(private_key)
		.map_err(|_| anyhow::anyhow!("解密失败"))?;
	let mut decrypted_aes_key = vec![0; private_key.min_output_size()];
	// Match WebCrypto RSA-OAEP: SHA-256 for both OAEP and MGF1, empty label.
	// AWS-LC replaces the timing-vulnerable RustCrypto rsa implementation.
	let decrypted_aes_key = private_key
		.decrypt(
			&OAEP_SHA256_MGF1SHA256,
			&aes_key,
			&mut decrypted_aes_key,
			None,
		)
		.map_err(|_| anyhow::anyhow!("解密失败"))?;
	// The frontend wraps the Base64 text of the 256-bit AES key, not raw bytes.
	let key = general_purpose::STANDARD.decode(decrypted_aes_key)?;
	let decryptor = Aes256CbcDec::new_from_slices(&key, &iv)
		.map_err(|_| anyhow::anyhow!("解密失败"))?;
	let plain_text = decryptor
		.decrypt_padded_mut::<Pkcs7>(&mut cipher_text)
		.map_err(|_| anyhow::anyhow!("解密失败"))?;
	Ok(std::str::from_utf8(plain_text)?.to_owned())
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde::Deserialize;

	#[derive(Deserialize)]
	struct Fixture {
		name: String,
		private_key: String,
		aes_key: String,
		cipher_text: String,
		iv: String,
		plaintext: Option<String>,
	}

	#[actix_web::test]
	async fn decrypts_frontend_payloads_and_rejects_invalid_payloads() {
		let fixtures: Vec<Fixture> =
			serde_json::from_str(include_str!("../../tests/fixtures/e2ee.json"))
				.unwrap();
		for fixture in fixtures {
			let result = decrypt_e2ee(
				&fixture.aes_key,
				&fixture.cipher_text,
				&fixture.iv,
				&fixture.private_key,
			)
			.await;
			if let Some(expected) = fixture.plaintext {
				assert_eq!(result.unwrap(), expected, "{}", fixture.name);
			} else {
				assert_eq!(
					result.unwrap_err().to_string(),
					"解密失败",
					"{}",
					fixture.name
				);
			}
		}
	}
}
