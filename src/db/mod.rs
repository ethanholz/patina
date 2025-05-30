use anyhow::Result;
use sqlx::SqlitePool;

pub async fn initialize(url: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
