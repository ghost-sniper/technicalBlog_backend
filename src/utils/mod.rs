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
	println!("BASE64解码");
	let cipher_text = general_purpose::STANDARD.decode(cipher_text_b64)?;
	let iv = general_purpose::STANDARD.decode(iv_b64)?;
	let aes_key = general_purpose::STANDARD.decode(aes_key_b64)?;
	println!("BASE64解码后AES_KEY长度:{}", aes_key.len());
	println!("初始化私钥");
	let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)?;
	let decrypted_aes_key = private_key.decrypt(Oaep::new::<Sha256>(), &aes_key)?;
	let dd = general_purpose::STANDARD.decode(&decrypted_aes_key)?;
	println!("BASE64再解码后AES_KEY长度：{}", dd.len());
	println!("创建原地解码空间");
	let mut buf = cipher_text.clone();
	// let pt_len = buf.len();
	println!("初始化AES解密器");
	// println!("解密后AES_KEY长度：{}", decrypted_aes_key.len());
	println!("IV长度：{}", iv.len());
	let key = GenericArray::from_slice(&dd);
	let iv = GenericArray::from_slice(&iv);
	println!("创建AES解密器");
	let decryptor = Aes256CbcDec::new(key, iv);
	println!("解密数据");
	let plain_text = decryptor.decrypt_padded_mut::<Pkcs7>(&mut buf).unwrap();
	println!("{}", String::from_utf8_lossy(&plain_text));
	Ok(String::from_utf8_lossy(&plain_text).to_string())
}
