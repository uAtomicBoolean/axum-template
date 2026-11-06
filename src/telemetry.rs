//! Logging with tracing. The level is set through RUST_LOG, for example:
//! `RUST_LOG=axum_template=debug,tower_http=info,sqlx=debug,info`

use tracing_subscriber::EnvFilter;

pub fn init() {
    let default = concat!(env!("CARGO_CRATE_NAME"), "=debug,tower_http=info,info");
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
