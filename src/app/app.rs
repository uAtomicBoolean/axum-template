use crate::app::config::Config;
use sea_orm::DatabaseConnection;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct App {
    db: Option<DatabaseConnection>,
    config: Arc<Config>,
}

impl App {
    pub fn build() -> anyhow::Result<Self> {
        Ok(App {
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
}
