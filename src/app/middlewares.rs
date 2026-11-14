use std::time::Duration;

use axum::Router;
use axum::http::header::{
    CONTENT_SECURITY_POLICY, HeaderName, HeaderValue, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS,
    X_FRAME_OPTIONS,
};
use axum::http::{Response, StatusCode};
use tower::ServiceBuilder;
use tower_http::LatencyUnit;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::compression::CompressionLayer;
use tower_http::set_header::HeaderMetadata;
use tower_http::set_header::response::SetMultipleResponseHeadersLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;

const HANDLER_TIMEOUT: Duration = Duration::from_secs(15);

/// How long the response body may stall between two frames before the
/// transfer is aborted. The clock restarts on every frame, so a long
/// download is fine as long as it keeps moving.
const RESPONSE_BODY_IDLE_TIMEOUT: Duration = Duration::from_secs(30);

/// Wraps `router` in the application middleware stack.
///
/// Call it last, once every route *and* the fallback are registered:
/// [`Router::layer`] only wraps what the router already holds.
pub fn apply<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let stack = ServiceBuilder::new()
        .layer(trace_layer())
        // Using `if_not_present` allow each route to define their own headers.
        .layer(SetMultipleResponseHeadersLayer::if_not_present(
            security_headers(),
        ))
        // Turns a panic into a 500 instead of a dropped connection. Below the
        // trace and header layers so that 500 is logged and carries the
        // headers, above everything that could panic.
        .layer(CatchPanicLayer::new())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            HANDLER_TIMEOUT,
        ));

    router.layer(stack)
}

/// One `INFO` line per request (`method`, `uri` and `version` on the span,
/// status and latency on the response).
fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Millis),
        )
        .on_failure(
            DefaultOnFailure::new()
                .level(Level::ERROR)
                .latency_unit(LatencyUnit::Millis),
        )
}

fn security_headers<B>() -> Vec<HeaderMetadata<Response<B>>> {
    [
        // Content types are taken literally, never guessed from the bytes.
        (X_CONTENT_TYPE_OPTIONS, "nosniff"),
        // No framing, for the browsers that predate `frame-ancestors`.
        (X_FRAME_OPTIONS, "DENY"),
        // Cross-origin navigations only leak the origin, not the full URL.
        (REFERRER_POLICY, "strict-origin-when-cross-origin"),
        // Everything comes from this origin. Inline `<style>`, inline
        // `style="…"` and inline `<script>` are refused: templates have to
        // link their assets from `/static`.
        (
            CONTENT_SECURITY_POLICY,
            "default-src 'self'; img-src 'self' data:; object-src 'none'; \
             base-uri 'self'; form-action 'self'; frame-ancestors 'none'",
        ),
        // A window this app opens, or that opens it, gets its own browsing
        // context group.
        (
            HeaderName::from_static("cross-origin-opener-policy"),
            "same-origin",
        ),
    ]
    .into_iter()
    .map(|(name, value)| (name, HeaderValue::from_static(value)).into())
    .collect()
}
