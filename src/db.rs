//! SQLite connection through SeaORM, and migrations on startup.

use std::path::Path;

use anyhow::Context;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};

pub async fn connect(url: &str) -> anyhow::Result<DatabaseConnection> {
    create_parent_dir(url)?;

    let mut options = ConnectOptions::new(url);
    options
        .max_connections(8)
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    let db = Database::connect(options)
        .await
        .with_context(|| format!("could not connect to the database `{url}`"))?;

    // WAL mode: concurrent reads and writes, a good fit for a web server.
    db.execute_unprepared("PRAGMA journal_mode = WAL;").await?;

    Migrator::up(&db, None)
        .await
        .context("failed to apply the migrations")?;
    tracing::info!("database ready");

    Ok(db)
}

/// `mode=rwc` creates the SQLite file, but not its parent folder.
fn create_parent_dir(url: &str) -> anyhow::Result<()> {
    let Some(rest) = url
        .strip_prefix("sqlite://")
        .or_else(|| url.strip_prefix("sqlite:"))
    else {
        return Ok(());
    };

    let path = rest.split('?').next().unwrap_or_default();
    if path.is_empty() || path.starts_with(":memory:") {
        return Ok(());
    }

    if let Some(parent) = Path::new(path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("could not create the folder {}", parent.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_url_of_another_engine_is_left_alone() {
        assert!(create_parent_dir("postgres://localhost/app").is_ok());
        assert!(create_parent_dir("mysql://localhost/app").is_ok());
        assert!(create_parent_dir("").is_ok());
    }

    #[test]
    fn an_in_memory_database_needs_no_folder() {
        assert!(create_parent_dir("sqlite::memory:").is_ok());
        assert!(create_parent_dir("sqlite://:memory:").is_ok());
    }

    #[test]
    fn a_database_at_the_root_of_the_working_directory_needs_no_folder() {
        assert!(create_parent_dir("sqlite://app.db?mode=rwc").is_ok());
        assert!(create_parent_dir("sqlite:app.db").is_ok());
    }

    #[test]
    fn a_missing_parent_folder_is_created() {
        let root = std::env::temp_dir().join(format!(
            "axum-template-db-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);

        // Forward slashes: this is a URL, not a path, even on Windows.
        let url = format!(
            "sqlite://{}/nested/app.db?mode=rwc",
            root.display().to_string().replace('\\', "/")
        );
        create_parent_dir(&url).expect("the parent folder should be created");
        assert!(root.join("nested").is_dir());

        let _ = std::fs::remove_dir_all(&root);
    }
}
