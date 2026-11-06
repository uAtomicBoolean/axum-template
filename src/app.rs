//! Application wiring: shared state, router and middleware.

use std::{any::Any, sync::Arc, time::Duration};

use axum::{
    Router,
    body::Body,
    extract::FromRef,
    http::{HeaderName, HeaderValue, Response, StatusCode, header},
};
use sea_orm::DatabaseConnection;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::{CatchPanicLayer, ResponseForPanic},
    compression::CompressionLayer,
    services::ServeDir,
    set_header::SetResponseHeaderLayer,
    timeout::{ResponseBodyTimeoutLayer, TimeoutLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::{config::Config, db, error, routes};

/// How long a handler may take to produce its response, before the client
/// gets a 408.
/// Doesn't cover a body's stream, only the handler's response.
/// This is done by the `BODY_STALL_TIMEOUT` const.
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

/// How long a *response* body may go quiet between two chunks.
///
/// The deadline resets on every chunk, so a slow but progressing transfer
/// still completes and a large download over a thin link is not cut off.
///
/// Request bodies are deliberately not covered: a slow uploader is the job of
/// rate limiting or of the reverse proxy in front, and bounding it here needs
/// a body-type dance that costs more in clarity than it buys. What does bound
/// the process either way is `SHUTDOWN_GRACE` in `main.rs`.
const BODY_STALL_TIMEOUT: Duration = Duration::from_secs(15);

/// `Cache-Control` for `/static`.
///
/// `no-cache` means "revalidate", not "do not store": the browser still keeps
/// the file and `ServeDir`'s `ETag` turns the next request into a cheap 304.
/// It stays conservative because `main.css` has no content hash in its name,
/// so a real `max-age` would serve a stale stylesheet after a deploy. Put a
/// hash in the filename first, then raise this to a year.
const STATIC_CACHE_CONTROL: &str = "no-cache";

/// What the pages actually load: the stylesheet and the favicon from this
/// origin, and nothing else — no inline style, no script, no framing.
///
/// Widen it when you add a CDN, analytics or an inline `<script>`; an inline
/// one needs a nonce or a hash, never `'unsafe-inline'`.
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; \
     img-src 'self' data:; \
     style-src 'self'; \
     script-src 'self'; \
     object-src 'none'; \
     base-uri 'self'; \
     form-action 'self'; \
     frame-ancestors 'none'";

/// State shared by every request. Cloning is cheap
/// (the connection pool and the config live behind `Arc`s).
///
/// To add a shared dependency (HTTP client, cache…): add a field here,
/// then expose it through `Ctx` (src/ctx.rs).
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Arc<Config>,
}

impl AppState {
    /// Connects to the database and applies pending migrations.
    pub async fn init(config: Config) -> anyhow::Result<Self> {
        let db = db::connect(&config.database_url).await?;
        Ok(Self {
            db,
            config: Arc::new(config),
        })
    }
}

/// Also allows extracting `State<DatabaseConnection>` directly in a
/// handler, if you would rather not use `Ctx`.
impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

pub fn build(state: AppState) -> Router {
    let static_dir = state.config.static_dir.clone();

    routes::router()
        .fallback(routes::not_found)
        .method_not_allowed_fallback(routes::method_not_allowed)
        .nest_service(
            "/static",
            ServiceBuilder::new()
                .layer(header_layer(header::CACHE_CONTROL, STATIC_CACHE_CONTROL))
                .service(ServeDir::new(static_dir)),
        )
        // A `ServiceBuilder` reads in the order requests travel: the first
        // layer is the outermost, the last one sits against the handler.
        .layer(
            ServiceBuilder::new()
                // Outermost, so every request is logged whatever happens
                // below — a timeout and a caught panic included.
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                // Above the compression, so the stall is measured on the bytes
                // that actually go out on the wire.
                .layer(ResponseBodyTimeoutLayer::new(BODY_STALL_TIMEOUT))
                .layer(CompressionLayer::new())
                // Above the two layers below, so their responses carry the
                // headers too.
                .layer(header_layer(header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
                .layer(header_layer(
                    header::REFERRER_POLICY,
                    "strict-origin-when-cross-origin",
                ))
                .layer(header_layer(header::X_FRAME_OPTIONS, "DENY"))
                .layer(header_layer(
                    header::CONTENT_SECURITY_POLICY,
                    CONTENT_SECURITY_POLICY,
                ))
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    RESPONSE_TIMEOUT,
                ))
                // Innermost: a panic in a handler becomes the same 500 page
                // as any other server error — which the `TraceLayer` above
                // still gets to log — instead of a dropped connection the
                // client sees as a reset.
                .layer(CatchPanicLayer::custom(PanicPage)),
        )
        .with_state(state)
}

/// One response header with a fixed value, on every response.
///
/// `Strict-Transport-Security` is deliberately not among them: it belongs to
/// whatever terminates TLS in front of the app, and this template serves
/// plain HTTP in development.
fn header_layer(name: HeaderName, value: &'static str) -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(name, HeaderValue::from_static(value))
}

/// Turns a caught panic into the 500 page of `error.rs`.
///
/// `tower-http`'s own handler answers with the `text/plain` line
/// `Service panicked`, which would make a panic the one server error that
/// does not look like the others. It also logs the panic, so this does too.
#[derive(Clone, Copy)]
struct PanicPage;

impl ResponseForPanic for PanicPage {
    type ResponseBody = axum::body::Body;

    fn response_for_panic(&mut self, err: Box<dyn Any + Send + 'static>) -> Response<Body> {
        let cause = err
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| err.downcast_ref::<&str>().copied())
            .unwrap_or("panic payload of an unknown type");
        tracing::error!(cause, "a handler panicked");

        error::server_error()
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    use super::*;

    /// The whole router, on a throwaway database.
    ///
    /// Plain `sqlite::memory:` would not do: every connection of the pool
    /// gets its own empty database, so the migrations would be invisible to
    /// the next request. A *named* in-memory database with `cache=shared` is
    /// one database for the whole pool, and `name` keeps these tests — which
    /// cargo runs in parallel — out of each other's way.
    async fn router(name: &str) -> Router {
        let config = Config {
            addr: "127.0.0.1:0".parse().unwrap(),
            database_url: format!("sqlite:file:{name}?mode=memory&cache=shared"),
            static_dir: "static".into(),
            default_lang: &crate::i18n::LANGUAGES[0],
        };
        build(AppState::init(config).await.expect("the test state"))
    }

    /// A `GET`, answered by the router with all its middleware in place.
    async fn get(name: &str, uri: &str, language: &str) -> (StatusCode, String) {
        let response = router(name)
            .await
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .header(header::ACCEPT_LANGUAGE, language)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("the router should answer");

        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("the body should be readable");
        (status, String::from_utf8_lossy(&body).into_owned())
    }

    #[tokio::test]
    async fn the_home_page_renders() {
        let (status, body) = get("home", "/", "en").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Welcome"), "{body}");
    }

    #[tokio::test]
    async fn an_unknown_url_renders_the_translated_404() {
        let (status, body) = get("unknown-url", "/nope", "fr").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.contains("Page introuvable"), "{body}");
        // In the layout, not the bare 5xx page.
        assert!(body.contains("site-header"), "{body}");
    }

    #[tokio::test]
    async fn a_resource_a_handler_cannot_find_renders_the_same_404() {
        // `/lang/xx` goes through `AppError::NotFound`, not the router's
        // fallback. Both have to produce the one 404 page.
        let (status, body) = get("unknown-lang", "/lang/xx", "fr").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.contains("Page introuvable"), "{body}");
        assert!(body.contains("site-header"), "{body}");
    }

    #[tokio::test]
    async fn every_response_carries_the_security_headers() {
        let response = router("headers")
            .await
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .expect("the router should answer");

        let headers = response.headers();
        assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
        assert_eq!(headers[header::X_FRAME_OPTIONS], "DENY");
        assert_eq!(
            headers[header::REFERRER_POLICY],
            "strict-origin-when-cross-origin"
        );
        assert!(headers.contains_key(header::CONTENT_SECURITY_POLICY));
    }

    #[tokio::test]
    async fn the_wrong_method_renders_a_translated_405_with_allow() {
        let response = router("method")
            .await
            .oneshot(
                Request::builder()
                    .uri("/notes/1/delete") // exists, but only for POST
                    .header(header::ACCEPT_LANGUAGE, "fr")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("the router should answer");

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        // `Allow` is what a client actually needs from a 405.
        assert_eq!(response.headers()[header::ALLOW], "POST");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("the body should be readable");
        let body = String::from_utf8_lossy(&body);
        assert!(body.contains("Méthode non autorisée"), "{body}");
        assert!(body.contains("site-header"), "{body}");
    }

    #[tokio::test]
    async fn a_panicking_handler_becomes_the_500_page() {
        // The panic prints its own line to stderr while this runs: that is
        // `CatchPanicLayer` working, not a failing test. It only works
        // because the release profile unwinds — `panic = "abort"` would kill
        // the process here instead.
        let (status, body) = get("panic", "/panic-on-purpose", "en").await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(body.contains("500"), "{body}");
        assert!(body.contains("Internal Server Error"), "{body}");
        // And not tower-http's bare `Service panicked` line.
        assert!(!body.contains("Service panicked"), "{body}");
    }

    #[tokio::test]
    async fn static_files_are_told_to_revalidate() {
        let response = router("cache")
            .await
            .oneshot(
                Request::builder()
                    .uri("/static/favicon.svg")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("the router should answer");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
        // The security headers reach the nested service too.
        assert_eq!(
            response.headers()[header::X_CONTENT_TYPE_OPTIONS],
            "nosniff"
        );
    }

    #[tokio::test]
    async fn a_hostile_referer_redirects_to_the_root() {
        // End to end over `is_local_path`: `/\evil.com` is read as
        // `//evil.com` by browsers, so it must never reach the `Location`.
        let response = router("referer")
            .await
            .oneshot(
                Request::builder()
                    .uri("/lang/fr")
                    .header(header::REFERER, r"/\evil.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("the router should answer");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::LOCATION], "/");
    }

    #[tokio::test]
    async fn the_language_cookie_is_not_reachable_from_a_script() {
        let response = router("cookie")
            .await
            .oneshot(
                Request::builder()
                    .uri("/lang/fr")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("the router should answer");

        let cookie = response.headers()[header::SET_COOKIE]
            .to_str()
            .expect("the cookie should be printable");
        assert!(cookie.starts_with("lang=fr;"), "{cookie}");
        assert!(cookie.contains("HttpOnly"), "{cookie}");
        assert!(cookie.contains("SameSite=Lax"), "{cookie}");
        assert!(cookie.contains("Path=/"), "{cookie}");
    }
}
