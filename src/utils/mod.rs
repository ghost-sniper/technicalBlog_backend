use aes::cipher::{
	BlockDecryptMut, KeyIvInit, block_padding::Pkcs7, generic_array::GenericArray,
};
use base64::{Engine as _, engine::general_purpose};
use cbc::Decryptor;
use pkcs8::DecodePrivateKey;
use rsa::{Oaep, RsaPrivateKey};
use sha2::Sha256;

type Aes256CbcDec = Decryptor<aes::Aes256>;

pub async fn decrypt_e2ee(
	aes_key_b64: &str,
	cipher_text_b64: &str,
	iv_b64: &str,
	private_key_pem: &str,
) -> anyhow::Result<String> {
	let cipher_text = general_purpose::STANDARD.decode(cipher_text_b64)?;
	let iv = general_purpose::STANDARD.decode(iv_b64)?;
	let aes_key = general_purpose::STANDARD.decode(aes_key_b64)?;
	let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)?;
	let decrypted_aes_key = private_key.decrypt(Oaep::new::<Sha256>(), &aes_key)?;
	let dd = general_purpose::STANDARD.decode(&decrypted_aes_key)?;
	let mut buf = cipher_text.clone();
	let key = GenericArray::from_slice(&dd);
	let iv = GenericArray::from_slice(&iv);
	let decryptor = Aes256CbcDec::new(key, iv);
	let plain_text = decryptor.decrypt_padded_mut::<Pkcs7>(&mut buf).unwrap();
	println!("{}", String::from_utf8_lossy(&plain_text));
	Ok(String::from_utf8_lossy(&plain_text).to_string())
}
