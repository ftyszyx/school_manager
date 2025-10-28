use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Duration, Utc};
use crate::apis::auth_middleware::Claims;
use crate::core::error::AppError;
use crate::core::config::JwtConfig;

pub fn create_jwt(user_id: i32, role_ids: Vec<i32>, jwt_config: &JwtConfig) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(jwt_config.expire_days as i64))
        .expect("valid timestamp")
        .timestamp();
    let claims = Claims {
        user_id,
        role_ids,
        exp: expiration as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_config.secret.as_ref()),
    )
    .map_err(|e| AppError::Message(format!("JWT encoding failed: {}", e)))
}
