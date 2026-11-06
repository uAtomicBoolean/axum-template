//! JSON API example, alongside the HTML pages.

use crate::entities::note;
use crate::prelude::*;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/notes", get(list))
}

#[derive(Serialize)]
struct NoteDto {
    id: i32,
    title: String,
    body: String,
    created_at: String,
}

impl From<note::Model> for NoteDto {
    fn from(note: note::Model) -> Self {
        Self {
            id: note.id,
            title: note.title,
            body: note.body,
            created_at: note.created_at.to_rfc3339(),
        }
    }
}

async fn list(ctx: Ctx) -> AppResult<Json<Vec<NoteDto>>> {
    let notes = Note::find()
        .order_by_desc(note::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    Ok(Json(notes.into_iter().map(NoteDto::from).collect()))
}
