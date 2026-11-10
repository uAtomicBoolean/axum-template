# Adding to the template

[← README](../README.md)

- [The `Ctx` context](#the-ctx-context)
- [Adding a page](#adding-a-page)
- [Request and response bodies](#request-and-response-bodies)
- [Adding a table](#adding-a-table)

## The `Ctx` context

Every handler can ask for a `ctx: Ctx`. axum builds it on each request,
and it gives access to everything:

| Field / method             | Contents                                   |
|----------------------------|--------------------------------------------|
| `ctx.db`                   | SeaORM connection (`&ctx.db` in queries)   |
| `ctx.config`               | application configuration                  |
| `ctx.t("key")`             | translation in the language of the request |
| `ctx.t_arg("key", "n", 3)` | translation with a variable (plurals)      |
| `ctx.lang()`               | code of the current language (`en`, `fr`)  |
| `ctx.datetime(date)`       | date formatted for the language            |
| `ctx.is_active("/notes")`  | true when on that section (menu)           |

Every page struct holds a `ctx: Ctx` field. The `layouts/base.html`
layout uses it (title, menu, language), and so do your own templates:
`{{ ctx.t("notes-title") }}`.

## Adding a page

1. Create `src/routes/contact.rs`:

   ```rust
   use crate::prelude::*;

   pub fn routes() -> Router<AppState> {
       Router::new().route("/contact", get(index))
   }

   #[derive(Template, WebTemplate)]
   #[template(path = "contact.html")]
   struct ContactPage {
       ctx: Ctx,
   }

   async fn index(ctx: Ctx) -> AppResult<ContactPage> {
       Ok(ContactPage { ctx })
   }
   ```

2. Register it in `src/routes/mod.rs`: `mod contact;` then
   `.merge(contact::routes())`.

3. Create `templates/contact.html`:

   ```html
   {% extends "layouts/base.html" %}
   {% block content %}
   <h1>{{ ctx.t("contact-title") }}</h1>
   {% endblock %}
   ```

4. Add `contact-title = …` to every `locales/<language>/main.ftl`, see
   [Translations](i18n.md).

`src/routes/notes.rs` shows a full CRUD (form, validation, redirect) and
`src/routes/api.rs` a JSON route.

## Request and response bodies

`src/payloads/` holds every struct that exists only to cross the HTTP
boundary: what a handler deserializes from a request, and what it serializes
into a response.

Nothing of the sort goes into `src/entities/`, which stays free of
`Serialize`, `Deserialize` and of anything else that knows about HTTP: that
folder is regenerated from the database (see
[Adding a table](#adding-a-table)), so anything written there is lost on the
next run.

The sub-modules of `payloads` are private, and `src/payloads/mod.rs`
re-exports every type:

```rust
// src/payloads/mod.rs
mod note;

pub use note::{NoteForm, NoteResponse};
```

A routes module then imports the type, never the module:

```rust
use crate::entities::note;         // the entity: note::Model, note::Column
use crate::payloads::NoteResponse; // the JSON shape
```

Importing `crate::payloads::note` instead would collide with
`crate::entities::note`, since two modules cannot share a name in the same
file. Re-exporting at the root of `payloads` is what keeps that collision
from ever happening, the same way `src/entities/prelude.rs` exposes
`Note` instead of `note::Entity`.

A response type keeps its fields private and is built by a
`From<note::Model>` impl, so a handler cannot assemble one the database never
produced. A form or request body needs `pub` fields as soon as a template
reads them back — `templates/notes/index.html` refills `NoteForm` after a
validation error.

## Adding a table

The project uses the [schema-first workflow](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/) which
is the default of sea-orm.

Create a new migration using [sea-orm-cli](https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/):

```sh
sea-orm-cli migrate generate create_article   # creates migration/src/m…_create_article.rs
```

Write the migration in the `migration` crate (check out the one for `note`). Then run the migrations:

```sh
cargo run -p migration -- up  # or simply restart the app
```

Unlike the application, the migration CLI has no built-in fallback: it reads
`DATABASE_URL` from the environment or from `.env`, and panics when it is not
set.

The CLI lives behind the `cli` feature of the `migration` crate, on by
default, so that command needs nothing extra. The server depends on the crate
with `default-features = false`, which keeps clap and `sea-orm-cli` out of
the release binary — they are a command-line parser, and the server has no
command line. Building the whole workspace in one go (`cargo build
--workspace`) unifies the features again and puts them back in, so build the
binary you ship with `cargo build --release` alone.

Then generate the entity following the newly created schema:

```sh
sea-orm-cli generate entity -u "sqlite://data/app.db" -o src/entities
```

This command regenerates `src/entities/` (including `mod.rs` and
`prelude.rs`) from the database. You can also write the entity by hand,
following `src/entities/note.rs`.

Other migration commands: `status`,
`down`, `fresh` (`cargo run -p migration -- --help`).
