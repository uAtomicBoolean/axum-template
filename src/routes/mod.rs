//! Route registry.
//!
//! To add a section to the site:
//! 1. create `src/routes/my_section.rs` with a `pub fn routes() -> Router<AppState>` function;
//! 2. declare the module below (`mod my_section;`);
//! 3. add `.merge(my_section::routes())` to `router()`.

use crate::prelude::*;

mod api;
mod home;
mod lang;
mod notes;

pub fn router() -> Router<AppState> {
    let router = Router::new()
        .merge(home::routes())
        .merge(notes::routes())
        .merge(lang::routes())
        .merge(api::routes());

    // Compiled only by `cargo test`. It is how the router tests of `app.rs`
    // reach `CatchPanicLayer` through the real middleware stack, without a
    // handler that could ever panic in a running server.
    #[cfg(test)]
    let router = router.route("/panic-on-purpose", get(panic_on_purpose));

    router
}

#[cfg(test)]
async fn panic_on_purpose() -> StatusCode {
    panic!("on purpose, from a test route");
}

/// The one 404 page. Served by the fallback below for an unknown URL, and by
/// `AppError::NotFound` for a resource a handler could not find.
#[derive(Template, WebTemplate)]
#[template(path = "404.html")]
pub struct NotFoundPage {
    ctx: Ctx,
}

impl NotFoundPage {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }
}

/// 404 page for every unknown URL.
pub async fn not_found(ctx: Ctx) -> (StatusCode, NotFoundPage) {
    (StatusCode::NOT_FOUND, NotFoundPage::new(ctx))
}

/// 405 page: the URL exists, the method does not. axum keeps adding the
/// `Allow` header, which is what a client actually needs here.
#[derive(Template, WebTemplate)]
#[template(path = "405.html")]
pub struct MethodNotAllowedPage {
    ctx: Ctx,
}

pub async fn method_not_allowed(ctx: Ctx) -> (StatusCode, MethodNotAllowedPage) {
    (StatusCode::METHOD_NOT_ALLOWED, MethodNotAllowedPage { ctx })
}
