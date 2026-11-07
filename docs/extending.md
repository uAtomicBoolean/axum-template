# Adding to the template

[← README](../README.md)

- [The `Ctx` context](#the-ctx-context)
- [Adding a page](#adding-a-page)
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
