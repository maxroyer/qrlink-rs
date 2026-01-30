#[path = "repository/link_repository.rs"]
mod link_repository;

pub use link_repository::LinkRepository;

use sqlx::sqlite::SqlitePool;

/// Database pool type alias.
pub type DbPool = SqlitePool;

/// Initialize the database pool and run migrations.
pub async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
    // Ensure the data directory exists
    if let Some(path) = database_url.strip_prefix("sqlite:")
        && let Some(parent) = std::path::Path::new(path).parent()
    {
        std::fs::create_dir_all(parent).map_err(sqlx::Error::Io)?;
    }

    // Connect with create-if-missing flag
    let pool = SqlitePool::connect(&format!("{}?mode=rwc", database_url)).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
