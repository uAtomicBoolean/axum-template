use std::env;

pub struct Config {
    pub app_addr: String,
    pub database_url: String,
    pub static_dir: String,
    pub default_lang: String,
    pub rust_log: String,
}

impl Config {
    pub fn init() -> Self {
        Config {
            app_addr: load_env_var("APP_ADDR"),
            database_url: load_env_var("DATABASE_URL"),
            static_dir: load_env_var("STATIC_DIR"),
            default_lang: load_env_var("DEFAULT_LANG"),
            rust_log: load_env_var("RUST_LOG"),
        }
    }
}

fn load_env_var<T: From<String>>(name: &str) -> T {
    env::var(name)
        .expect(format!("Could not load `{name}` from environment.").as_str())
        .into()
}
