use crate::{
	models::user::{UserLoginParams, UserVo},
	services::{LoginError, LoginResult},
};
use regex::Regex;

const CRYPTO_KEY: &[u8; 32] = b"02294834837484739437366283472329";

pub fn login_user(params: UserLoginParams) -> LoginResult<UserVo> {
	let account = params.account.as_deref().unwrap_or("");
	let password = params.password.as_deref().unwrap_or("");
	let encrypted_password =
		crate::utils::encrypt(password.as_bytes(), CRYPTO_KEY).unwrap();

	Err(LoginError::WrongCredentials)
}
