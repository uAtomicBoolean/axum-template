//! Configuration read from environment variables (and `.env`).

use std::{
    env, fmt,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::i18n::{self, Language};

#[derive(Clone)]
pub struct Config {
    /// `APP_ADDR`, defaults to 127.0.0.1:3000
    pub addr: SocketAddr,
    /// `DATABASE_URL`, defaults to sqlite://data/app.db?mode=rwc
    pub database_url: String,
    /// `STATIC_DIR`, defaults to ./static
    pub static_dir: PathBuf,
    /// `DEFAULT_LANG`, defaults to en
    pub default_lang: &'static Language,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let addr: SocketAddr = var("APP_ADDR", "127.0.0.1:3000")
            .parse()
            .context("invalid APP_ADDR (example: 127.0.0.1:3000)")?;

        let lang = var("DEFAULT_LANG", "en");
        let default_lang = i18n::find(&lang)
            .with_context(|| format!("DEFAULT_LANG `{lang}` is not a supported language"))?;

        let static_dir = PathBuf::from(var("STATIC_DIR", "static"));
        check_static_dir(&static_dir)?;

        Ok(Self {
            addr,
            database_url: var("DATABASE_URL", "sqlite://data/app.db?mode=rwc"),
            static_dir,
            default_lang,
        })
    }
}

/// Written by hand, and not derived, so that `database_url` cannot reach a
/// log.
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("addr", &self.addr)
            .field("database_url", &"<redacted>")
            .field("static_dir", &self.static_dir)
            .field("default_lang", &self.default_lang)
            .finish()
    }
}

/// Refusing to start is the point: a folder that is not there serves every
/// page unstyled, and the other invalid values all fail here too rather than
/// at the first request.
fn check_static_dir(dir: &Path) -> anyhow::Result<()> {
    anyhow::ensure!(
        dir.is_dir(),
        "STATIC_DIR `{}` is not an existing folder (a relative path is \
         resolved from the working directory)",
        dir.display()
    );
    Ok(())
}

/// Reads an environment variable, falling back to `default`.
fn var(key: &str, default: &str) -> String {
    pick(env::var(key).ok(), default)
}

/// A variable that is set but blank counts as absent.
///
/// `env::var` only reports an *absent* variable, so a `STATIC_DIR=` left in a
/// `.env` or an environment variable a deployment filled with nothing used to
/// reach `ServeDir::new("")` — which serves the whole working directory.
/// Values are trimmed too, so a stray space in a `.env` does not break parsing.
///
/// Split out of `var` so it can be tested without touching the environment of
/// the test process, which every test shares.
fn pick(value: Option<String>, default: &str) -> String {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blank_value_falls_back_to_the_default() {
        assert_eq!(pick(None, "static"), "static");
        assert_eq!(pick(Some(String::new()), "static"), "static");
        assert_eq!(pick(Some("   ".to_owned()), "static"), "static");
        assert_eq!(pick(Some("\t\n".to_owned()), "static"), "static");
    }

    #[test]
    fn a_real_value_wins_and_is_trimmed() {
        assert_eq!(pick(Some("public".to_owned()), "static"), "public");
        assert_eq!(pick(Some("  public \n".to_owned()), "static"), "public");
    }

    #[test]
    fn an_existing_static_dir_is_accepted() {
        // Relative to the crate root, which is where cargo runs the tests.
        assert!(check_static_dir(Path::new("static")).is_ok());
        assert!(check_static_dir(Path::new("templates")).is_ok());
    }

    #[test]
    fn a_missing_static_dir_is_refused() {
        let error = check_static_dir(Path::new("no-such-folder"))
            .expect_err("a folder that is not there must stop the startup");
        assert!(error.to_string().contains("STATIC_DIR"), "{error}");
    }

    #[test]
    fn a_file_is_not_a_static_dir() {
        assert!(check_static_dir(Path::new("Cargo.toml")).is_err());
    }

    #[test]
    fn debug_never_prints_the_database_url() {
        let config = Config {
            addr: "127.0.0.1:3000".parse().unwrap(),
            database_url: "postgres://admin:hunter2@db.internal/app".to_owned(),
            static_dir: "static".into(),
            default_lang: &i18n::LANGUAGES[0],
        };

        let printed = format!("{config:?}");
        assert!(!printed.contains("hunter2"), "{printed}");
        assert!(!printed.contains("db.internal"), "{printed}");
        assert!(printed.contains("<redacted>"), "{printed}");
        // The rest is still useful when debugging.
        assert!(printed.contains("127.0.0.1:3000"), "{printed}");
    }
}
