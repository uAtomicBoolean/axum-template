use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use sea_orm::DatabaseConnection;

use crate::app::AppState;
use crate::app::config::Config;

#[derive(Clone)]
pub struct Context {
    pub db: DatabaseConnection,

    pub config: Arc<Config>,
    pub path: String,
}

impl FromRequestParts<AppState> for Context {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(db) = state.db.clone() else {
            tracing::error!("the application state has no database connection");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };

        Ok(Context {
            db,
            config: Arc::clone(&state.config),
            path: parts.uri.path().to_owned(),
        })
    }
}
