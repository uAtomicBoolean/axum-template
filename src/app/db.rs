use anyhow::Context;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};

pub async fn connect(
    database_url: &str,
    logging_level: log::LevelFilter,
) -> anyhow::Result<DatabaseConnection> {
    let mut options = ConnectOptions::new(database_url);
    options
        .max_connections(8)
        .sqlx_logging(true)
        .sqlx_logging_level(logging_level);

    let db = Database::connect(options)
        .await
        .with_context(|| format!("could not connect to the database `{database_url}`"))?;

    db.execute_unprepared("PRAGMA journal_mode = WAL;").await?;

    Migrator::up(&db, None)
        .await
        .context("Failed to apply database migrations.")?;

    tracing::info!("Database ready!");

    Ok(db)
}
