mod api;
mod auth;
mod config;
mod database;
mod models;
mod repository;
mod websocket;

use axum::routing::{get, post};
use axum::Router;
use config::AppConfig;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
    pub ws_state: Arc<websocket::WsState>,
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "code": 0,
        "message": "ok",
        "data": {
            "status": "ok",
            "version": "0.1.0",
        }
    }))
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "LogHub API",
        version = "0.1.0",
        description = "LogHub application log management platform API"
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "logs", description = "Log ingestion and query endpoints"),
    ),
    modifiers(&SecurityAddon),
    paths(
        api::handlers::auth::login,
        api::handlers::auth::me,
        api::handlers::log::ingest,
        api::handlers::log::list,
        api::handlers::log::services,
    ),
    components(schemas(
        models::user::LoginRequest,
        models::user::LoginResponse,
        models::user::UserInfo,
        models::log::IngestLogRequest,
    ))
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Arc::new(AppConfig::load()?);
    let pool = database::create_pool(&config.database.url).await?;
    database::run_migrations(&pool).await?;
    database::seed_admin_user(&pool, &config.admin).await?;

    let ws_state = Arc::new(websocket::WsState::new());

    let state = AppState {
        pool,
        config: config.clone(),
        ws_state: ws_state.clone(),
    };

    let frontend_dist = config.frontend.dist.clone();

    let openapi = ApiDoc::openapi();

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/auth/login", post(api::handlers::auth::login))
        .route("/api/auth/me", get(api::handlers::auth::me))
        .route("/api/logs", get(api::handlers::log::list).post(api::handlers::log::ingest))
        .route("/api/logs/services", get(api::handlers::log::services))
        .route("/api/logs/ws", get(websocket::ws_handler))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .fallback_service(
            ServeDir::new(&frontend_dist)
                .fallback(ServeFile::new(format!("{}/index.html", frontend_dist))),
        );

    let addr = SocketAddr::new(config.server.host.parse()?, config.server.port);
    tracing::info!("Server starting on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
