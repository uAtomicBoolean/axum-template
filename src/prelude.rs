//! Everything you need to write a routes module: `use crate::prelude::*;`
#![allow(unused_imports)]

pub use askama::Template;
pub use askama_web::WebTemplate;
pub use axum::{
    Json, Router,
    extract::{Form, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, post, put},
};
pub use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
pub use serde::{Deserialize, Serialize};

pub use crate::{
    app::AppState,
    ctx::Ctx,
    entities::prelude::*,
    error::{AppError, AppResult},
};
