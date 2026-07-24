use ring::{
	aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey},
	error::Unspecified,
	rand::{SecureRandom, SystemRandom},
};

pub fn encrypt(plain_text: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Unspecified> {
	let rng = SystemRandom::new();
	//使用密钥初始化加密上下文
	let unbound_key =
		UnboundKey::new(&AES_256_GCM, key).expect("unboundKey初始化失败");
	let key = LessSafeKey::new(unbound_key);
	//生成一个12字节的随机Nonce（数字只使用1次）
	// AES_GCM推荐使用12字节的Nonce
	let mut nonce_bytes = [0u8; 12];
	rng.fill(&mut nonce_bytes).unwrap();
	let nonce = ring::aead::Nonce::assume_unique_for_key(nonce_bytes);
	// 准备加密明文
	let mut in_out = plain_text.to_vec();

	//执行加密
	key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)?;

	let mut result = nonce_bytes.to_vec();
	result.append(&mut in_out);
	Ok(result)
}

pub fn decrypt(cipher_text: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Unspecified> {
	let (nonce_bytes, encrypted) = cipher_text.split_at(12);
	let nonce = Nonce::assume_unique_for_key(nonce_bytes.try_into().unwrap());

	let unbound_key =
		UnboundKey::new(&AES_256_GCM, key).expect("unboundKey初始化失败");
	let key = LessSafeKey::new(unbound_key);

	let mut in_out = encrypted.to_vec();
	let plain_text = key.open_in_place(nonce, Aad::empty(), &mut in_out)?;
	Ok(plain_text.to_vec())
}
#[cfg(test)]
mod tests {
	#[test]
	fn test_encrypt_decrypt() {
		let key: [u8; 32] = *b"01234567890123456789012345678901";
		let plain_text = b"Hello, World!";
		let encrypted = super::encrypt(plain_text, &key).unwrap();
		let decrypted = super::decrypt(&encrypted, &key).unwrap();
		assert_eq!(plain_text, decrypted.as_slice());
	}
}
