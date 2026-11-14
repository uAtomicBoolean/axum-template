use crate::app;
use crate::app::config::Config;
use axum::Router;
use sea_orm::DatabaseConnection;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub(super) db: Option<DatabaseConnection>,
    pub(super) config: Arc<Config>,
}

impl AppState {
    pub fn build() -> anyhow::Result<Self> {
        Ok(AppState {
            db: None,
            config: Arc::new(Config::init()?),
        })
    }

    pub fn init_logging(&self) {
        let filter = EnvFilter::new(self.config.rust_log.as_str());
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .init();
    }

    pub async fn init_database(&mut self) -> anyhow::Result<()> {
        let logging_level = match std::env::var("PROFILE").as_deref() {
            Ok("release") => log::LevelFilter::Info,
            _ => log::LevelFilter::Debug,
        };

        let db = crate::app::db::connect(self.config.database_url.as_str(), logging_level).await?;
        self.db = Some(db);
        Ok(())
    }

    pub fn get_server_addr(&self) -> SocketAddr {
        self.config.app_addr.clone()
    }

    /// Used to avoid using directly the middleware imports in other files.
    pub fn apply_middlewares<S>(&self, router: Router<S>) -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        app::middlewares::apply(router)
    }
}
