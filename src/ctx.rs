//! `Ctx`: the request context, extracted automatically by axum.
//!
//! A handler that takes `ctx: Ctx` as a parameter has access to the
//! database (`ctx.db`), the configuration, translation and the current
//! path. Templates receive the same `ctx`: `{{ ctx.t("home-title") }}`.

use std::{borrow::Borrow, convert::Infallible, fmt::Write, sync::Arc};

use axum::{extract::FromRequestParts, http::request::Parts};
use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;

use crate::{
    app::AppState,
    config::Config,
    i18n::{self, I18n, Language, ToFluent},
};

#[derive(Debug, Clone)]
pub struct Ctx {
    pub db: DatabaseConnection,
    #[allow(dead_code)] // available to handlers
    pub config: Arc<Config>,
    pub i18n: I18n,
    /// Path of the request, e.g. `/notes`
    pub path: String,
}

impl FromRequestParts<AppState> for Ctx {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
            config: state.config.clone(),
            i18n: I18n::negotiate(&parts.headers, state.config.default_lang),
            path: parts.uri.path().to_owned(),
        })
    }
}

/// Shortcuts used in handlers and templates.
impl Ctx {
    /// `{{ ctx.t("home-title") }}`
    pub fn t(&self, key: &str) -> String {
        self.i18n.t(key)
    }

    /// `{{ ctx.t_arg("home-note-count", "count", note_count) }}`
    pub fn t_arg(&self, key: &str, name: &str, value: impl ToFluent) -> String {
        self.i18n.t_arg(key, name, value)
    }

    /// Code of the current language, e.g. `en`.
    pub fn lang(&self) -> &'static str {
        self.i18n.code()
    }

    pub fn languages(&self) -> &'static [Language] {
        i18n::LANGUAGES
    }

    /// True if the current page is `prefix` or one of its sub-pages.
    pub fn is_active(&self, prefix: &str) -> bool {
        if prefix == "/" {
            return self.path == "/";
        }
        self.path == prefix
            || self
                .path
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('/'))
    }

    /// Date formatted with the `format-datetime` key of the current language.
    ///
    /// The value is printed as it is stored, in UTC: there is no timezone to
    /// convert to, since the server never learns the visitor's.
    pub fn datetime(&self, value: impl Borrow<DateTime<Utc>>) -> String {
        let value = value.borrow();
        let format = self.t("format-datetime");
        let mut out = String::new();
        // An invalid format in a .ftl file must not bring the page down.
        if write!(out, "{}", value.format(&format)).is_err() {
            out = value.to_rfc3339();
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::DatabaseConnection;

    use super::*;

    /// A context with no database behind it: enough for everything that only
    /// reads the path or the language.
    fn ctx(path: &str, lang: &'static Language) -> Ctx {
        Ctx {
            // `Default` is the disconnected state: these tests never query.
            db: DatabaseConnection::default(),
            config: Arc::new(Config {
                addr: "127.0.0.1:3000".parse().unwrap(),
                database_url: "sqlite::memory:".to_owned(),
                static_dir: "static".into(),
                default_lang: &i18n::LANGUAGES[0],
            }),
            i18n: I18n::new(lang),
            path: path.to_owned(),
        }
    }

    fn at(path: &str) -> Ctx {
        ctx(path, &i18n::LANGUAGES[0])
    }

    #[test]
    fn the_root_is_active_only_on_the_root() {
        assert!(at("/").is_active("/"));
        assert!(!at("/notes").is_active("/"));
        assert!(!at("/notes/1/delete").is_active("/"));
    }

    #[test]
    fn a_section_is_active_on_its_subpages() {
        assert!(at("/notes").is_active("/notes"));
        assert!(at("/notes/1").is_active("/notes"));
        assert!(at("/notes/1/delete").is_active("/notes"));
    }

    #[test]
    fn a_section_does_not_match_a_longer_name() {
        // The trap a `starts_with` would fall into.
        assert!(!at("/notesarchive").is_active("/notes"));
        assert!(!at("/notes-archive").is_active("/notes"));
        assert!(!at("/").is_active("/notes"));
    }

    #[test]
    fn datetime_uses_the_format_of_the_language() {
        let when = DateTime::parse_from_rfc3339("2026-11-07T08:05:00Z")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(
            ctx("/", &i18n::LANGUAGES[0]).datetime(when),
            "Nov 7, 2026, 08:05 UTC"
        );
        assert_eq!(
            ctx("/", &i18n::LANGUAGES[1]).datetime(when),
            "07/11/2026 à 08:05 UTC"
        );
    }

    #[test]
    fn datetime_says_which_timezone_it_is_printing() {
        // The value is stored and rendered in UTC, and the server never
        // learns the visitor's timezone, so leaving that implicit shows the
        // wrong time to most of the world without saying so.
        let when = DateTime::parse_from_rfc3339("2026-11-07T23:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        for lang in i18n::LANGUAGES {
            let printed = ctx("/", lang).datetime(when);
            assert!(printed.contains("23:30"), "{}: {printed}", lang.code);
            assert!(printed.contains("UTC"), "{}: {printed}", lang.code);
        }
    }
}
