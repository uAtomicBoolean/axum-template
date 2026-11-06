//! Migration CLI: `cargo run -p migration -- <command>`
//! (up, down, status, fresh, refresh, reset). Reads DATABASE_URL from `.env`.

use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
