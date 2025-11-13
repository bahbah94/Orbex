use anyhow::Result;
use sqlx::postgres::PgPool;
use tracing::info;

pub async fn init_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    info!("✅ Connected to database");
    Ok(pool)
}

// Deprecated: Use `just db-setup` instead
// pub async fn run_migrations(pool: &PgPool) -> Result<()> {
//     // Read and execute the migrations SQL file
//     let sql = include_str!("../../db/migrations/001_create_trades.sql");

//     // Split by statements and execute
//     for statement in sql.split(';').filter(|s| !s.trim().is_empty()) {
//         sqlx::query(statement)
//             .execute(pool)
//             .await?;
//     }

//     info!("✅ Migrations completed");
//     Ok(())
// }
