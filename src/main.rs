mod app;
mod config;
mod ctx;
mod db;
mod entities;
mod error;
mod i18n;
mod prelude;
mod routes;
mod telemetry;

use std::{future::IntoFuture, time::Duration};

use anyhow::Context;
use tokio::{net::TcpListener, sync::oneshot};

/// How long the connections still open get to finish, once the shutdown
/// signal has arrived.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(20);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `.env` is optional: every variable has a default value.
    dotenvy::dotenv().ok();
    telemetry::init();

    let config = config::Config::from_env()?;
    let addr = config.addr;

    let state = app::AppState::init(config).await?;
    let router = app::build(state);

    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("could not listen on {addr}"))?;
    tracing::info!("server started on {addr}");

    // The signal future tells the server to start draining, and opens the
    // grace period at the same moment.
    let (draining, drained) = oneshot::channel();
    let server = axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            let _ = draining.send(());
        })
        .into_future();

    tokio::select! {
        result = server => result.context("the server stopped unexpectedly")?,
        () = grace_period(drained) => tracing::warn!(
            grace = ?SHUTDOWN_GRACE,
            "connections still open after the grace period: closing anyway"
        ),
    }

    Ok(())
}

/// Resolves `SHUTDOWN_GRACE` after the shutdown signal, and never before it.
async fn grace_period(signalled: oneshot::Receiver<()>) {
    // An `Err` here means the sender was dropped, which only happens once the
    // shutdown future has run: either way, the signal has been given.
    let _ = signalled.await;
    tokio::time::sleep(SHUTDOWN_GRACE).await;
}

/// Graceful shutdown on Ctrl+C (and SIGTERM on Unix).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("could not listen for Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("could not listen for SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutting down the server");
}
