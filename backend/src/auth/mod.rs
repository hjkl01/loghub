use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct AuthError {
    pub code: i32,
    pub message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({
            "code": self.code,
            "message": self.message,
        }));
        (StatusCode::UNAUTHORIZED, body).into_response()
    }
}

pub fn create_token(
    user_id: i64,
    username: &str,
    role: &str,
    secret: &str,
    expires_in: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        role: role.to_string(),
        exp: (chrono::Utc::now().timestamp() + expires_in) as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub struct AuthenticatedUser {
    pub id: i64,
    pub username: String,
    pub role: String,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AuthError {
                code: 10001,
                message: "Missing authorization header".to_string(),
            })?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AuthError {
                code: 10001,
                message: "Invalid authorization format".to_string(),
            })?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.config.auth.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError {
            code: 10002,
            message: "Invalid or expired token".to_string(),
        })?;

        Ok(AuthenticatedUser {
            id: token_data.claims.sub,
            username: token_data.claims.username,
            role: token_data.claims.role,
        })
    }
}
