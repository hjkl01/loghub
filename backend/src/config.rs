use anyhow::Result;
use config::Config;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expires_in: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdminConfig {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FrontendConfig {
    pub dist: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub admin: AdminConfig,
    pub frontend: FrontendConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let cfg = Config::builder()
            .add_source(
                config::Environment::default()
                    .prefix("LOG")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;
        Ok(cfg.try_deserialize()?)
    }
}
