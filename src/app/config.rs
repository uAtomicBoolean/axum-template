use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

pub struct Config {
    pub app_addr: SocketAddr,
    pub database_url: String,
    pub static_dir: PathBuf,
    pub default_lang: String,
    pub rust_log: String,
}

impl Config {
    pub fn init() -> anyhow::Result<Self> {
        let app_addr: SocketAddr = load_env_var::<String>("APP_ADDR").parse()?;
        let static_dir = PathBuf::from(load_env_var::<String>("STATIC_DIR"));
        anyhow::ensure!(
            static_dir.is_dir(),
            "`STATIC_DIR` \"{}\" is not an existing folder.",
            static_dir.display()
        );

        Ok(Config {
            app_addr,
            database_url: load_env_var("DATABASE_URL"),
            static_dir,
            default_lang: load_env_var("DEFAULT_LANG"),
            rust_log: load_env_var("RUST_LOG"),
        })
    }
}

fn load_env_var<T: From<String>>(name: &str) -> T {
    env::var(name)
        .expect(format!("Could not load `{name}` from environment.").as_str())
        .into()
}
