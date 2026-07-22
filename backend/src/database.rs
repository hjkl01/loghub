use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::AdminConfig;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

pub async fn seed_admin_user(pool: &PgPool, admin_config: &AdminConfig) -> Result<()> {
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE username = $1",
    )
    .bind(&admin_config.username)
    .fetch_one(pool)
    .await?;

    if existing > 0 {
        tracing::info!("Admin user '{}' already exists, skipping seed", admin_config.username);
        return Ok(());
    }

    let password_hash = bcrypt::hash(&admin_config.password, bcrypt::DEFAULT_COST)?;

    sqlx::query(
        "INSERT INTO users (username, password_hash, role) VALUES ($1, $2, 'admin')",
    )
    .bind(&admin_config.username)
    .bind(&password_hash)
    .execute(pool)
    .await?;

    tracing::info!("Admin user '{}' created successfully", admin_config.username);
    Ok(())
}
