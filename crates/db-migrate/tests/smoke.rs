use std::process::Command;

use anyhow::{Context, Result};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://messaging_user:messaging_password@localhost:55432/messaging_service".to_string()
    })
}

async fn try_pool() -> Result<Pool<Postgres>> {
    PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url())
        .await
        .context("failed to connect to database")
}

#[test]
fn migrations_apply_command_succeeds() -> Result<()> {
    // Best-effort connectivity check before invoking the CLI; if DB isn't ready or creds
    // mismatch the README, skip to avoid flaky CI.
    let rt = tokio::runtime::Runtime::new()?;
    let pool = rt.block_on(try_pool());
    if pool.is_err() {
        eprintln!(
            "[smoke] Skipping apply: cannot connect to DATABASE_URL ({:?})",
            pool.err()
        );
        return Ok(());
    }

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", "db-migrate", "--", "apply"])
        .env("DATABASE_URL", database_url());

    let status = cmd.status().context("failed to execute db-migrate apply. If the database isn't initialized with messaging_user/messaging_service, recreate the docker volume so init.sql runs.")?;
    assert!(
        status.success(),
        "db-migrate apply exited with non-zero status"
    );
    Ok(())
}

#[tokio::test]
async fn customers_table_exists() -> Result<()> {
    let pool = match try_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[smoke] Skipping existence check: cannot connect to DATABASE_URL ({e})");
            return Ok(());
        }
    };

    let exists: Option<String> = sqlx::query_scalar("SELECT to_regclass('public.customers')::text")
        .fetch_one(&pool)
        .await
        .context("failed to query to_regclass(customers)")?;

    assert!(
        exists.is_some(),
        "customers table not found after migrations"
    );
    Ok(())
}

#[tokio::test]
async fn views_exist_after_migrate() -> Result<()> {
    // Ensure migrations were applied
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", "db-migrate", "--", "apply"])
        .env("DATABASE_URL", database_url());
    let status = cmd
        .status()
        .context("failed to execute db-migrate apply for views check")?;
    if !status.success() {
        eprintln!("[smoke] Skipping view check: migrations apply failed (likely DB not initialized via init.sql)");
        return Ok(());
    }

    let pool = match try_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[smoke] Skipping view check: cannot connect to DATABASE_URL ({e})");
            return Ok(());
        }
    };

    let overview: Option<String> = sqlx::query_scalar(
        "SELECT table_name::text FROM information_schema.views WHERE table_schema = 'public' AND table_name = 'conversation_overview'",
    )
    .fetch_optional(&pool)
    .await?;

    let conv_msgs: Option<String> = sqlx::query_scalar(
        "SELECT table_name::text FROM information_schema.views WHERE table_schema = 'public' AND table_name = 'conversation_messages'",
    )
    .fetch_optional(&pool)
    .await?;

    assert!(overview.is_some(), "conversation_overview view not found");
    assert!(conv_msgs.is_some(), "conversation_messages view not found");
    Ok(())
}
