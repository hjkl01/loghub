use axum::extract::State;
use axum::Json;
use serde_json::Value;

use crate::auth::{create_token, AuthenticatedUser};
use crate::models::user::LoginRequest;
use crate::repository::user_repo;
use crate::AppState;

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = Value),
        (status = 401, description = "Invalid credentials", body = Value),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    let user = user_repo::find_by_username(&state.pool, &req.username)
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": 20001,
                    "message": "Internal server error",
                })),
            )
        })?;

    let user = user.ok_or_else(|| {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "code": 10003,
                "message": "Invalid username or password",
            })),
        )
    })?;

    let valid = bcrypt::verify(&req.password, &user.password_hash).map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": 20001,
                "message": "Internal server error",
            })),
        )
    })?;

    if !valid {
        return Err((
            axum::http::StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "code": 10003,
                "message": "Invalid username or password",
            })),
        ));
    }

    let token = create_token(
        user.id,
        &user.username,
        &user.role,
        &state.config.auth.jwt_secret,
        state.config.auth.jwt_expires_in,
    )
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": 20001,
                "message": "Token creation failed",
            })),
        )
    })?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "ok",
        "data": {
            "token": token,
            "user": {
                "id": user.id,
                "username": user.username,
                "role": user.role,
            }
        }
    })))
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current user info", body = Value),
        (status = 401, description = "Unauthorized", body = Value),
    )
)]
pub async fn me(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
) -> Json<Value> {
    Json(serde_json::json!({
        "code": 0,
        "message": "ok",
        "data": {
            "id": user.id,
            "username": user.username,
            "role": user.role,
        }
    }))
}
