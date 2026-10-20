use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use thiserror::Error;

use crate::handlers::presentation::UnifiedResponse;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LoginError {
	#[error("{0}")]
	InvalidRequest(&'static str),
	#[error("用户不存在")]
	UserNotFound,
	#[error("用户账号已被冻结，请联系管理员")]
	AccountFrozen,
	#[error("用户密码错误")]
	WrongPassword,
	#[error("登录加密参数无效，请刷新页面后重试")]
	InvalidEncryptedPayload,
	#[error("登录密钥已失效，请刷新页面后重试")]
	InvalidLoginKey,
	#[error("登录服务暂时不可用，请稍后重试")]
	Internal,
}

impl LoginError {
	pub fn code(&self) -> i32 {
		match self {
			Self::InvalidRequest(_) => 40001,
			Self::InvalidEncryptedPayload => 40002,
			Self::InvalidLoginKey => 40003,
			Self::UserNotFound => 40101,
			Self::WrongPassword => 40102,
			Self::AccountFrozen => 40301,
			Self::Internal => 50001,
		}
	}
}

impl ResponseError for LoginError {
	fn status_code(&self) -> StatusCode {
		match self {
			Self::InvalidRequest(_)
			| Self::InvalidEncryptedPayload
			| Self::InvalidLoginKey => StatusCode::BAD_REQUEST,
			Self::UserNotFound | Self::WrongPassword => StatusCode::UNAUTHORIZED,
			Self::AccountFrozen => StatusCode::FORBIDDEN,
			Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}

	fn error_response(&self) -> HttpResponse {
		HttpResponse::build(self.status_code()).json(UnifiedResponse {
			status: "error".to_owned(),
			code: self.code(),
			message: Some(self.to_string()),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use actix_web::body::to_bytes;

	#[actix_web::test]
	async fn errors_have_distinct_codes_and_json_messages() {
		for (error, status, code, message) in [
			(LoginError::UserNotFound, 401, 40101, "用户不存在"),
			(
				LoginError::AccountFrozen,
				403,
				40301,
				"用户账号已被冻结，请联系管理员",
			),
			(LoginError::WrongPassword, 401, 40102, "用户密码错误"),
			(
				LoginError::InvalidRequest("未传入账号"),
				400,
				40001,
				"未传入账号",
			),
			(
				LoginError::InvalidEncryptedPayload,
				400,
				40002,
				"登录加密参数无效，请刷新页面后重试",
			),
			(
				LoginError::InvalidLoginKey,
				400,
				40003,
				"登录密钥已失效，请刷新页面后重试",
			),
			(
				LoginError::Internal,
				500,
				50001,
				"登录服务暂时不可用，请稍后重试",
			),
		] {
			let response = error.error_response();
			assert_eq!(response.status().as_u16(), status);
			let body = to_bytes(response.into_body()).await.unwrap();
			let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
			assert_eq!(
				body,
				serde_json::json!({"status": "error", "code": code, "message": message})
			);
		}
	}
}
