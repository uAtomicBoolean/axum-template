//! The entities representations in the HTTP requests, both in and out.
//! This prevent the application to sending private data or to only accept required data like a
//! form.
//!
//! Sub-modules stay private and every type is re-exported here, so a routes
//! module imports the type and never a module whose name would collide with
//! its entity (`crate::entities::note`).

mod note;

pub use note::{NoteForm, NoteResponse};
