use chrono::{DateTime, Utc};
use sqlx::Row;
use url::Url;
use uuid::Uuid;

use crate::domain::{Link, ShortCode};
use crate::error::{AppError, AppResult};

use super::DbPool;

/// Repository for link persistence operations.
#[derive(Clone)]
pub struct LinkRepository {
    pool: DbPool,
}

impl LinkRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Create a new link. Returns the created link or an error if short_code already exists.
    pub async fn create(
        &self,
        id: Uuid,
        short_code: &ShortCode,
        target_url: &Url,
        created_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    ) -> AppResult<Link> {
        let id_str = id.to_string();
        let short_code_str = short_code.as_str();
        let target_url_str = target_url.to_string();
        let created_at_str = created_at.to_rfc3339();
        let expires_at_str = expires_at.map(|e| e.to_rfc3339());

        sqlx::query(
            r#"
            INSERT INTO links (id, short_code, target_url, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id_str)
        .bind(short_code_str)
        .bind(&target_url_str)
        .bind(&created_at_str)
        .bind(&expires_at_str)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.message().contains("UNIQUE constraint failed") {
                    return AppError::ShortCodeExhausted;
                }
            }
            AppError::Database(e)
        })?;

        Ok(Link {
            id,
            short_code: short_code.clone(),
            target_url: target_url.clone(),
            created_at,
            expires_at,
        })
    }

    /// Find a link by its short code.
    pub async fn find_by_short_code(&self, short_code: &str) -> AppResult<Option<Link>> {
        let row = sqlx::query(
            r#"
            SELECT id, short_code, target_url, created_at, expires_at
            FROM links
            WHERE short_code = ?
            "#,
        )
        .bind(short_code)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_link(row)?)),
            None => Ok(None),
        }
    }

    /// List all links (no filtering).
    pub async fn list_all(&self) -> AppResult<Vec<Link>> {
        let rows = sqlx::query(
            r#"
            SELECT id, short_code, target_url, created_at, expires_at
            FROM links
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|row| self.row_to_link(row)).collect()
    }

    /// Delete a link by its ID. Returns true if a link was deleted.
    pub async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let id_str = id.to_string();
        let result = sqlx::query("DELETE FROM links WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete all expired links.
    pub async fn delete_expired(&self) -> AppResult<u64> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            r#"
            DELETE FROM links
            WHERE expires_at IS NOT NULL AND expires_at < ?
            "#,
        )
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    fn row_to_link(&self, row: sqlx::sqlite::SqliteRow) -> AppResult<Link> {
        let id_str: String = row.get("id");
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| AppError::Internal(format!("Invalid UUID in database: {}", e)))?;

        let short_code: String = row.get("short_code");
        let target_url_str: String = row.get("target_url");
        let target_url = Url::parse(&target_url_str)
            .map_err(|e| AppError::Internal(format!("Invalid URL in database: {}", e)))?;

        let created_at_str: String = row.get("created_at");
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid datetime in database: {}", e)))?
            .with_timezone(&Utc);

        let expires_at_str: Option<String> = row.get("expires_at");
        let expires_at = expires_at_str
            .map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| AppError::Internal(format!("Invalid expiry datetime: {}", e)))
            })
            .transpose()?;

        Ok(Link {
            id,
            short_code: ShortCode::from_existing(short_code),
            target_url,
            created_at,
            expires_at,
        })
    }
}
