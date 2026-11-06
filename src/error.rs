//! Error type shared by every handler. `?` works directly on SeaORM,
//! askama and anyhow errors.

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use crate::{ctx::Ctx, routes::NotFoundPage};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Carries the request context so the page can be rendered in the
    /// visitor's language, exactly like the router's own fallback.
    #[error("resource not found")]
    NotFound(Ctx),

    #[error("database: {0}")]
    Db(#[from] sea_orm::DbErr),

    #[error("template: {0}")]
    Template(#[from] askama::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Last-resort page for a 5xx: no layout, no translation, so that it still
/// renders when i18n or the database is exactly what failed.
#[derive(Template)]
#[template(path = "error.html")]
struct ErrorPage {
    code: u16,
    reason: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // A 404 is not a failure of the application: it gets the real page,
        // in the layout and in the visitor's language, the same one the
        // router's fallback serves for an unknown URL.
        if let Self::NotFound(ctx) = self {
            return (StatusCode::NOT_FOUND, NotFoundPage::new(ctx)).into_response();
        }

        tracing::error!(error = %self, "error while handling the request");
        server_error()
    }
}

/// The 500 page, as a response.
///
/// Shared with the panic handler of `app.rs`, so that a panic caught by
/// `CatchPanicLayer` looks like any other server error instead of the bare
/// `text/plain` line `tower-http` would send. It logs nothing itself: each
/// caller knows what it is reporting.
pub fn server_error() -> Response {
    let status = StatusCode::INTERNAL_SERVER_ERROR;
    let page = ErrorPage {
        code: status.as_u16(),
        reason: status.canonical_reason().unwrap_or("Error"),
    };
    match page.render() {
        Ok(html) => (status, Html(html)).into_response(),
        // Nothing left to render with: the status alone still beats a
        // dropped connection.
        Err(_) => status.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use sea_orm::DbErr;

    use super::*;

    async fn body_of(response: Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("the body should be readable");
        String::from_utf8_lossy(&bytes).into_owned()
    }

    #[tokio::test]
    async fn a_database_error_renders_the_500_page() {
        let response = AppError::Db(DbErr::Custom("boom".into())).into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = body_of(response).await;
        assert!(body.contains("500"), "{body}");
        assert!(body.contains("Internal Server Error"), "{body}");
        // The last-resort page is standalone, so no layout and no `Ctx`.
        assert!(!body.contains("site-header"), "{body}");
    }

    #[tokio::test]
    async fn a_500_never_leaks_what_went_wrong() {
        // The cause goes to the logs, not to the visitor: a `DbErr` carries
        // table and column names, and sometimes the query.
        let response =
            AppError::Other(anyhow::anyhow!("connection to 10.0.0.7 refused")).into_response();

        let body = body_of(response).await;
        assert!(!body.contains("10.0.0.7"), "{body}");
        assert!(!body.contains("refused"), "{body}");
    }

    #[tokio::test]
    async fn the_shared_500_response_is_the_same_page() {
        // `app.rs` hands this to `CatchPanicLayer`; it has to match what a
        // handler's own error produces.
        let from_error = body_of(AppError::Db(DbErr::Custom("boom".into())).into_response()).await;
        let from_panic = body_of(server_error()).await;
        assert_eq!(from_error, from_panic);
    }
}
