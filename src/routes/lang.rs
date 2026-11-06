//! Language switching: `/lang/fr` stores the choice in a cookie,
//! then redirects back to the page the visitor came from.

use axum::http::{HeaderMap, Uri, header};

use crate::i18n;
use crate::prelude::*;

pub fn routes() -> Router<AppState> {
    Router::new().route("/lang/{code}", get(switch_lang))
}

async fn switch_lang(
    ctx: Ctx,
    Path(code): Path<String>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let lang = i18n::find(&code).ok_or_else(|| AppError::NotFound(ctx))?;

    // Keep only the path of the Referer, so we never redirect off-site.
    let back = headers
        .get(header::REFERER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<Uri>().ok())
        .map(|uri| uri.path().to_owned())
        .filter(|path| is_local_path(path))
        .unwrap_or_else(|| "/".to_owned());

    // `HttpOnly` costs nothing here — only the server reads this cookie — and
    // keeps it out of reach of any script a future XSS might run. Add
    // `Secure` behind TLS: it would stop the cookie working over the plain
    // HTTP this template serves in development.
    let cookie = format!(
        "{}={}; Path=/; Max-Age=31536000; SameSite=Lax; HttpOnly",
        i18n::COOKIE,
        lang.code
    );

    Ok(([(header::SET_COOKIE, cookie)], Redirect::to(&back)).into_response())
}

/// True for a path that keeps the visitor on this site.
///
/// `//host` and `/\host` are both read as protocol-relative URLs by browsers:
/// the URL standard treats `\` like `/` right after the first slash. Neither
/// may follow it, or the redirect leaves the site.
fn is_local_path(path: &str) -> bool {
    let mut chars = path.chars();
    chars.next() == Some('/') && !matches!(chars.next(), Some('/' | '\\'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_on_this_site_is_kept() {
        assert!(is_local_path("/"));
        assert!(is_local_path("/notes"));
        assert!(is_local_path("/notes/1/delete"));
        assert!(is_local_path("/a//b"));
    }

    #[test]
    fn a_protocol_relative_path_is_refused() {
        // Both of these are read as `//evil.com` — an URL on another host —
        // because the URL standard treats `\` like `/` after the first slash.
        assert!(!is_local_path("//evil.com"));
        assert!(!is_local_path(r"/\evil.com"));
        assert!(!is_local_path(r"/\\evil.com"));
        assert!(!is_local_path("//"));
    }

    #[test]
    fn anything_that_is_not_a_path_is_refused() {
        assert!(!is_local_path(""));
        assert!(!is_local_path("evil.com"));
        assert!(!is_local_path("https://evil.com"));
        assert!(!is_local_path(" /notes"));
    }

    #[test]
    fn a_percent_encoded_backslash_stays_a_path() {
        // `%5C` is not decoded before the URL is parsed, so this is a path on
        // this site and not an authority. Refusing it would be harmless but
        // wrong, and would hint the check works on decoded input.
        assert!(is_local_path("/%5Cevil.com"));
    }
}
