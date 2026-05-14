use anyhow::Context;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn build_pool(database_url: &str) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .with_context(|| "failed to connect to postgres")
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    let sql = include_str!("../../infra/db/init.sql");
    sqlx::raw_sql(sql)
        .execute(pool)
        .await
        .context("failed to run bootstrap sql")?;
    Ok(())
}
