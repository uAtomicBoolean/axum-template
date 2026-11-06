//! Internationalisation with Fluent (fluent-templates).
//!
//! The `locales/<language>/*.ftl` files are embedded in the binary at
//! compile time. To add a language: create `locales/<code>/main.ftl`,
//! then add it to `LANGUAGES`.

use std::{borrow::Cow, collections::HashMap};

use axum::http::{HeaderMap, header};
use fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader};

fluent_templates::static_loader! {
    static LOCALES = {
        locales: "./locales",
        // Language used when a key is missing from the requested one.
        fallback_language: "en",
        // Without this, Fluent wraps every variable in invisible Unicode characters.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

/// Name of the cookie that remembers the language choice.
pub const COOKIE: &str = "lang";

#[derive(Debug)]
pub struct Language {
    pub code: &'static str,
    pub label: &'static str,
}

/// Languages offered in the switcher.
pub const LANGUAGES: &[Language] = &[
    Language {
        code: "en",
        label: "English",
    },
    Language {
        code: "fr",
        label: "Français",
    },
];

/// Finds a supported language from a code (`en`, `en-US`, `FR`…).
pub fn find(code: &str) -> Option<&'static Language> {
    let primary = code.trim().split(['-', '_']).next()?;
    LANGUAGES
        .iter()
        .find(|lang| lang.code.eq_ignore_ascii_case(primary))
}

/// Best supported language from an `Accept-Language` header.
///
/// The header is a comma-separated list of `tag;q=weight` entries, where a
/// missing weight means 1 and `q=0` means "not acceptable". Browsers are free
/// to send them in any order, so the weights decide and not the position:
/// `fr;q=0.1, en;q=0.9` has to pick English.
fn from_accept_language(headers: &HeaderMap) -> Option<&'static Language> {
    let header = headers.get(header::ACCEPT_LANGUAGE)?.to_str().ok()?;

    let mut best: Option<(f32, &'static Language)> = None;
    for entry in header.split(',') {
        let mut parts = entry.split(';');

        let Some(lang) = parts.next().and_then(find) else {
            continue;
        };

        // A weight we cannot read counts as a rejection rather than as the
        // default of 1, so a malformed entry never outranks a sound one.
        let quality: f32 = parts
            .find_map(|part| part.trim().strip_prefix("q="))
            .map_or(1.0, |weight| weight.trim().parse().unwrap_or(0.0));

        if quality > 0.0 && best.is_none_or(|(highest, _)| quality > highest) {
            best = Some((quality, lang));
        }
    }

    best.map(|(_, lang)| lang)
}

/// Translator for a given language (one instance per request).
#[derive(Debug, Clone)]
pub struct I18n {
    lang: &'static Language,
    id: LanguageIdentifier,
}

impl I18n {
    pub fn new(lang: &'static Language) -> Self {
        Self {
            lang,
            id: lang
                .code
                .parse()
                .expect("invalid language code in LANGUAGES"),
        }
    }

    /// Picks the language: cookie, then the `Accept-Language` header, then the default.
    pub fn negotiate(headers: &HeaderMap, default: &'static Language) -> Self {
        let from_cookie = headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(';'))
            .filter_map(|pair| pair.trim().split_once('='))
            .filter(|(name, _)| *name == COOKIE)
            .find_map(|(_, value)| find(value));

        Self::new(
            from_cookie
                .or_else(|| from_accept_language(headers))
                .unwrap_or(default),
        )
    }

    pub fn code(&self) -> &'static str {
        self.lang.code
    }

    /// Translates a key. A missing key is returned as-is (and logged).
    pub fn t(&self, key: &str) -> String {
        LOCALES
            .try_lookup(&self.id, key)
            .unwrap_or_else(|| self.missing(key))
    }

    /// Translates a key with one variable: `t_arg("home-note-count", "count", 3)`.
    pub fn t_arg(&self, key: &str, name: &str, value: impl ToFluent) -> String {
        self.t_args(key, &[(name, value.to_fluent())])
    }

    /// Translates a key with several variables.
    pub fn t_args(&self, key: &str, args: &[(&str, FluentValue<'_>)]) -> String {
        let args: HashMap<Cow<'static, str>, FluentValue<'_>> = args
            .iter()
            .map(|(name, value)| (Cow::Owned((*name).to_owned()), value.clone()))
            .collect();

        LOCALES
            .try_lookup_with_args(&self.id, key, &args)
            .unwrap_or_else(|| self.missing(key))
    }

    fn missing(&self, key: &str) -> String {
        tracing::warn!(key, lang = self.lang.code, "missing translation");
        key.to_owned()
    }
}

/// Values usable as Fluent variables: numbers (for plurals) and strings,
/// by value or by reference.
pub trait ToFluent {
    fn to_fluent(&self) -> FluentValue<'static>;
}

macro_rules! numbers_to_fluent {
    ($($ty:ty),*) => {
        $(impl ToFluent for $ty {
            fn to_fluent(&self) -> FluentValue<'static> {
                FluentValue::from(*self)
            }
        })*
    };
}

numbers_to_fluent!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

impl ToFluent for str {
    fn to_fluent(&self) -> FluentValue<'static> {
        FluentValue::from(self.to_owned())
    }
}

impl ToFluent for String {
    fn to_fluent(&self) -> FluentValue<'static> {
        FluentValue::from(self.clone())
    }
}

impl<T: ToFluent + ?Sized> ToFluent for &T {
    fn to_fluent(&self) -> FluentValue<'static> {
        (**self).to_fluent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `LANGUAGES[0]` is the default used by these tests, as `DEFAULT_LANG`
    /// defaults to `en`.
    const DEFAULT: &Language = &LANGUAGES[0];

    fn accept(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT_LANGUAGE, value.parse().unwrap());
        headers
    }

    fn negotiated(headers: &HeaderMap) -> &'static str {
        I18n::negotiate(headers, DEFAULT).code()
    }

    #[test]
    fn find_accepts_regions_whitespace_and_case() {
        assert_eq!(find("en").map(|l| l.code), Some("en"));
        assert_eq!(find("EN").map(|l| l.code), Some("en"));
        assert_eq!(find("en-US").map(|l| l.code), Some("en"));
        assert_eq!(find("fr_FR").map(|l| l.code), Some("fr"));
        assert_eq!(find("  fr  ").map(|l| l.code), Some("fr"));
        assert!(find("de").is_none());
        assert!(find("").is_none());
        assert!(find("*").is_none());
    }

    #[test]
    fn the_weights_decide_and_not_the_order() {
        // The regression this guards: taking the first supported tag, which
        // served French to a browser asking for English nine times out of ten.
        assert_eq!(negotiated(&accept("fr;q=0.1, en;q=0.9")), "en");
        assert_eq!(negotiated(&accept("en;q=0.1, fr;q=0.9")), "fr");
        assert_eq!(negotiated(&accept("de, fr;q=0.8, en;q=0.9")), "en");
    }

    #[test]
    fn equal_weights_keep_the_order_of_the_header() {
        assert_eq!(negotiated(&accept("fr, en")), "fr");
        assert_eq!(negotiated(&accept("en, fr")), "en");
        assert_eq!(negotiated(&accept("fr;q=0.5, en;q=0.5")), "fr");
    }

    #[test]
    fn a_missing_weight_means_one() {
        assert_eq!(negotiated(&accept("fr;q=0.9, en")), "en");
        assert_eq!(negotiated(&accept("en;q=0.9, fr")), "fr");
    }

    #[test]
    fn a_zero_or_unreadable_weight_is_a_rejection() {
        assert_eq!(negotiated(&accept("fr;q=0, en")), "en");
        assert_eq!(negotiated(&accept("en;q=bogus, fr;q=0.5")), "fr");
        // Everything rejected falls through to the default.
        assert_eq!(negotiated(&accept("en;q=0, fr;q=0")), "en");
    }

    #[test]
    fn an_unsupported_or_absent_header_falls_back_to_the_default() {
        assert_eq!(negotiated(&accept("de, it")), "en");
        assert_eq!(negotiated(&accept("*")), "en");
        assert_eq!(negotiated(&accept("")), "en");
        assert_eq!(negotiated(&HeaderMap::new()), "en");
    }

    #[test]
    fn the_cookie_wins_over_the_header() {
        let mut headers = accept("en;q=1.0");
        headers.insert(header::COOKIE, "lang=fr".parse().unwrap());
        assert_eq!(negotiated(&headers), "fr");
    }

    #[test]
    fn an_unknown_cookie_does_not_shadow_the_header() {
        let mut headers = accept("fr");
        headers.insert(header::COOKIE, "lang=de; other=x".parse().unwrap());
        assert_eq!(negotiated(&headers), "fr");
    }

    /// The keys declared in one `locales/<code>/main.ftl`, as written at the
    /// start of a line. Good enough for a parity check, and it needs no
    /// Fluent introspection.
    fn keys(code: &str) -> std::collections::BTreeSet<String> {
        let path = format!("locales/{code}/main.ftl");
        let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        source
            .lines()
            .filter_map(|line| line.split_once(" ="))
            .filter(|(key, _)| {
                !key.is_empty() && key.chars().all(|c| c.is_ascii_lowercase() || c == '-')
            })
            .map(|(key, _)| key.to_owned())
            .collect()
    }

    #[test]
    fn every_language_declares_the_same_keys() {
        // Fluent falls back to English for a key a translation is missing, so
        // a forgotten key is invisible at runtime: nothing breaks, the page
        // just turns English mid-sentence. This is the only thing that notices.
        let reference = keys(LANGUAGES[0].code);
        assert!(
            !reference.is_empty(),
            "no keys found for the first language"
        );

        for lang in &LANGUAGES[1..] {
            let other = keys(lang.code);

            let missing: Vec<_> = reference.difference(&other).collect();
            assert!(missing.is_empty(), "`{}` is missing {missing:?}", lang.code);

            let extra: Vec<_> = other.difference(&reference).collect();
            assert!(
                extra.is_empty(),
                "`{}` has {extra:?}, absent from `{}`",
                lang.code,
                LANGUAGES[0].code
            );
        }
    }

    #[test]
    fn every_declared_language_is_usable() {
        for lang in LANGUAGES {
            // `I18n::new` expects this to parse, on every single request.
            assert!(
                lang.code.parse::<LanguageIdentifier>().is_ok(),
                "`{}` is not a language identifier",
                lang.code
            );
            // A `LANGUAGES` entry with no `locales/<code>/main.ftl` would
            // echo every key back instead of translating it.
            let i18n = I18n::new(lang);
            assert_ne!(
                i18n.t("nav-home"),
                "nav-home",
                "`{}` has no keys",
                lang.code
            );
        }
    }
}
