use crate::app::config::Config;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct App {
    config: Arc<Config>,
}

impl App {
    pub fn build() -> anyhow::Result<Self> {
        Ok(App {
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

    pub fn get_server_addr(&self) -> SocketAddr {
        self.config.app_addr.clone()
    }
}
