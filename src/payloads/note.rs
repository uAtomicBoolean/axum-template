use crate::entities::note;
use crate::prelude::{Deserialize, Serialize};


#[derive(Serialize)]
pub struct NoteResponse {
    id: i32,
    title: String,
    body: String,
    created_at: String,
}

impl From<note::Model> for NoteResponse {
    fn from(note: note::Model) -> Self {
        Self {
            id: note.id,
            title: note.title,
            body: note.body,
            created_at: note.created_at.to_rfc3339(),
        }
    }
}


#[derive(Debug, Default, Deserialize)]
pub struct NoteForm {
    pub title: String,
    #[serde(default)]
    pub body: String,
}
