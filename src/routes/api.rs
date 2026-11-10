//! JSON API example, alongside the HTML pages.

use crate::entities::note;
use crate::payloads::NoteResponse;
use crate::prelude::*;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/notes", get(list))
}

async fn list(ctx: Ctx) -> AppResult<Json<Vec<NoteResponse>>> {
    let notes = Note::find()
        .order_by_desc(note::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    Ok(Json(notes.into_iter().map(NoteResponse::from).collect()))
}
