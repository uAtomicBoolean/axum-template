//! Full CRUD example: list, creation with validation, deletion.

use chrono::Utc;

use crate::entities::note;
use crate::prelude::*;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/notes", get(index).post(create))
        .route("/notes/{id}/delete", post(remove))
}

#[derive(Template, WebTemplate)]
#[template(path = "notes/index.html")]
struct NotesPage {
    ctx: Ctx,
    notes: Vec<note::Model>,
    form: NoteForm,
    error: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct NoteForm {
    title: String,
    #[serde(default)]
    body: String,
}

async fn index(ctx: Ctx) -> AppResult<NotesPage> {
    let notes = all_notes(&ctx).await?;
    Ok(NotesPage {
        ctx,
        notes,
        form: NoteForm::default(),
        error: None,
    })
}

async fn create(ctx: Ctx, Form(form): Form<NoteForm>) -> AppResult<Response> {
    let title = form.title.trim();

    if title.is_empty() {
        // Render the form again with the error message (HTTP 422).
        let notes = all_notes(&ctx).await?;
        let error = Some(ctx.t("notes-error-title-required"));
        let page = NotesPage {
            ctx,
            notes,
            form,
            error,
        };
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, page).into_response());
    }

    let note = note::ActiveModel {
        title: Set(title.to_owned()),
        body: Set(form.body.trim().to_owned()),
        created_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    tracing::info!(id = note.id, "note created");

    // Post/Redirect/Get: refreshing the page does not resubmit the form.
    Ok(Redirect::to("/notes").into_response())
}

#[tracing::instrument(skip(ctx))]
async fn remove(ctx: Ctx, Path(id): Path<i32>) -> AppResult<Redirect> {
    let result = Note::delete_by_id(id).exec(&ctx.db).await?;
    if result.rows_affected == 0 {
        return Err(AppError::NotFound(ctx));
    }
    tracing::info!("note deleted");
    Ok(Redirect::to("/notes"))
}

async fn all_notes(ctx: &Ctx) -> Result<Vec<note::Model>, DbErr> {
    Note::find()
        .order_by_desc(note::Column::CreatedAt)
        .all(&ctx.db)
        .await
}
