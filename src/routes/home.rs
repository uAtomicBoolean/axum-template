//! Minimal example: one page, one query, one template.

use crate::prelude::*;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(index))
}

#[derive(Template, WebTemplate)]
#[template(path = "home.html")]
struct HomePage {
    ctx: Ctx,
    note_count: u64,
}

async fn index(ctx: Ctx) -> AppResult<HomePage> {
    let note_count = Note::find().count(&ctx.db).await?;
    Ok(HomePage { ctx, note_count })
}
