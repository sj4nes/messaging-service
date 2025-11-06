use std::{env, fs, io::Write, path::PathBuf};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};

// Point to the dedicated SQLx migrations directory that contains only .up/.down.sql files
static MIGRATIONS: Migrator = sqlx::migrate!("./migrations_sqlx");

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("apply") => apply().await,
        Some("new") => {
            let name = args.next().unwrap_or_else(|| "new_migration".to_string());
            create_new_migration_pair(&name)
        }
        Some("status") => status().await,
        _ => {
            eprintln!(
                "Usage:\n  db-migrate apply\n  db-migrate new <name>\n  db-migrate status\n\nENV:\n  DATABASE_URL  Postgres connection URL"
            );
            Ok(())
        }
    }
}

async fn apply() -> Result<()> {
    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL is required to apply migrations")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    println!("Applying migrations from ./migrations_sqlx ...");
    MIGRATIONS
        .run(&pool)
        .await
        .context("failed to apply migrations")?;
    println!("Migrations applied successfully.");
    Ok(())
}

async fn status() -> Result<()> {
    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL is required to get status")?;

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    let row = sqlx::query!(
        r#"SELECT current_database()::text AS db, current_user::text AS usr, inet_server_addr()::TEXT AS host, inet_server_port()::int AS port"#
    )
    .fetch_one(&pool)
    .await?;

    println!(
        "Connected to {}@{}:{} (db={})",
        row.usr.unwrap_or_else(|| "?".into()),
        row.host.unwrap_or_else(|| "?".into()),
        row.port.unwrap_or(0),
        row.db.unwrap_or_else(|| "?".into())
    );

    let applied = sqlx::query!(
        r#"SELECT version, description, success, installed_on::text AS "installed_on?: String" FROM _sqlx_migrations ORDER BY version"#
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    if applied.is_empty() {
        println!("No entries in _sqlx_migrations.");
    } else {
        println!("Applied migrations:");
        for r in applied {
            println!(
                "  {:>4}  {:<24}  success={}  at={}",
                r.version,
                r.description,
                r.success,
                r.installed_on.unwrap_or_else(|| "?".into())
            );
        }
    }

    Ok(())
}

fn create_new_migration_pair(name: &str) -> Result<()> {
    // Sanitize name: lowercase, replace spaces with underscores, keep alnum and _-
    let safe: String = name
        .chars()
        .map(|c| match c {
            'A'..='Z' => c.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' | '_' | '-' => c,
            ' ' => '_',
            _ => '_',
        })
        .collect();

    let ts = Utc::now().format("%Y%m%d%H%M%S");
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Create new migrations in the dedicated migrations_sqlx directory
    dir.push("migrations_sqlx");
    fs::create_dir_all(&dir).context("failed to create migrations directory")?;

    let up_name = format!("{}_{}.up.sql", ts, safe);
    let down_name = format!("{}_{}.down.sql", ts, safe);

    let up_path = dir.join(&up_name);
    let down_path = dir.join(&down_name);

    if up_path.exists() || down_path.exists() {
        bail!(
            "migration files already exist: {} / {}",
            up_path.display(),
            down_path.display()
        );
    }

    let up_tmpl = b"-- Write your UP migration here.\n";
    let down_tmpl = b"-- Write your DOWN migration here.\n";

    let mut fup = fs::File::create(&up_path)
        .with_context(|| format!("failed to create {}", up_path.display()))?;
    fup.write_all(up_tmpl)?;

    let mut fdown = fs::File::create(&down_path)
        .with_context(|| format!("failed to create {}", down_path.display()))?;
    fdown.write_all(down_tmpl)?;

    println!(
        "Created migrations:\n  {}\n  {}",
        up_path.display(),
        down_path.display()
    );
    Ok(())
}
