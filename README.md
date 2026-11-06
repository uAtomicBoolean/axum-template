# axum-template

An opinionated Rust web application template using axum.

> [!NOTE]
> The template comes with a notes example in `src/routes/notes.rs` — a full CRUD (form, validation,
> redirect) — and a JSON route in `src/routes/api.rs`. Both are there to be read and replaced, not
> shipped as they are.

- [Tech stack](#tech-stack)
- [Requirements](#requirements)
- [Getting started](#getting-started)
- [Project structure](#project-structure)
- [The `Ctx` context](#the-ctx-context)
- [Adding a page](#adding-a-page)
- [Adding a table](#adding-a-table)
- [Translations](#translations)
- [SCSS](#scss)
- [Middleware and security](#middleware-and-security)
- [Tests](#tests)
- [Logging](#logging)
- [Going to production](#going-to-production)
- [License](#license)

## Tech stack

The template comes with an opinionated tech stack that brings templating, i18n, database support, etc.

| Role           | Tool                                        |
|----------------|---------------------------------------------|
| Server / API   | axum 0.8                                    |
| HTML templates | askama 0.16 (+ askama_web)                  |
| Translation    | Fluent (fluent-templates)                   |
| Database       | SQLite with SeaORM 2                        |
| Logs           | tracing + tower-http                        |
| Styles         | SCSS compiled by Dart Sass on `cargo build` |

## Requirements

- Rust 1.88 or newer (`rustup update`)
- [Dart Sass](https://sass-lang.com/install), either in `PATH`
  (`sass --version`) or pointed at by `SASS_BIN` — see [SCSS](#scss)
- Optional: `cargo install sea-orm-cli@^2` (generate migrations and entities)
- Optional: `cargo install watchexec-cli` (automatic reload)

## Getting started

Rename the project by changing `name` in `Cargo.toml` and `RUST_LOG` in `.env`. The default log filter follows the crate
name automatically.

```sh
cp .env.example .env     # optional here, required by the migration CLI
cargo run
```

Then open <http://127.0.0.1:3000>. On the first run, the `data/app.db`
database is created and the migrations are applied.

Every variable has a default in `src/config.rs`, so the application runs
without a `.env`. The migration CLI is the one exception: it reads
`DATABASE_URL` from the environment and panics when it is missing, so copy the
file before reaching for `cargo run -p migration` — see
[Adding a table](#adding-a-table).

### Rust and SCSS hot reload

During development, to rebuild and restart on every change:

```sh
watchexec -r -e rs,html,ftl,scss,toml -- cargo run
```

To iterate on the CSS without recompiling the Rust, run
`sass --watch --load-path=styles styles/main.scss:static/css/main.css`
alongside it, then reload the page.

## Project structure

```
├── build.rs                  compiles styles/ → static/css/main.css (Dart Sass)
├── locales/<language>/*.ftl  Fluent translations (embedded in the binary)
├── migration/                SeaORM migrations (separate crate)
├── static/                   files served under /static
├── styles/                   SCSS sources
├── templates/                askama templates (embedded in the binary)
│   └── layouts/base.html     shared layout
└── src/
    ├── main.rs               startup, graceful shutdown
    ├── app.rs                AppState, router, middleware
    ├── ctx.rs                Ctx: the context available in every handler
    ├── prelude.rs            common imports for routes modules
    ├── config.rs             environment variables
    ├── db.rs                 SQLite connection + migrations on startup
    ├── error.rs              AppError / AppResult
    ├── i18n.rs               Fluent loading, language negotiation
    ├── telemetry.rs          tracing setup
    ├── entities/             SeaORM entities
    └── routes/               one file per section of the site
```

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

4. Add `contact-title = …` to every `locales/<language>/main.ftl`.

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

## Translations

- A missing key is displayed as-is and produces a warning in the logs.
  English is the fallback language (`fallback_language` in `src/i18n.rs`).
- Language negotiation: the `lang` cookie (set by `/lang/{code}`), then the
  browser's `Accept-Language` header, then `DEFAULT_LANG`. The header's `q=`
  weights decide, not the order they come in, so `fr;q=0.1, en;q=0.9` serves
  English.
- New language: create `locales/de/main.ftl` and add
  `Language { code: "de", label: "Deutsch" }` to `LANGUAGES` in `src/i18n.rs`.
- Plurals are handled by Fluent according to each language (in French, 0
  takes the singular):

  ```ftl
  home-note-count = { $count ->
      [one] You have { $count } note.
     *[other] You have { $count } notes.
  }
  ```

## SCSS

> [!NOTE]
> The template requires Dart Sass to be installed on the computer.
> This is an opiniated choice and is assumed.

`build.rs` runs `sass` on every build where `styles/` has changed (CSS is
compressed in `--release`).

By default it looks for Dart Sass in `PATH`. On Windows the batch wrappers
`sass.bat` (official archive) and `sass.cmd` (npm) are picked up too.

To use another copy, set `SASS_BIN` to either the executable or the folder
that contains it:

```sh
SASS_BIN=/opt/dart-sass/sass        # the executable
# or
SASS_BIN=/opt/dart-sass             # the folder holding it
```

`build.rs` reads `.env`, so that line can simply go there. Your shell and
`.cargo/config.toml` (`[env]`) work as well and take precedence over
`.env`. If `SASS_BIN` leads nowhere, the build says so and falls back to
`PATH`.

If you delete `static/css/main.css` by hand, touch a file in `styles/` so
that it gets regenerated.

## Middleware and security

`src/app.rs` wraps every route — static files included — in one
`ServiceBuilder`, which reads in the order requests travel: the first layer is
the outermost, the last one sits against the handler.

| Layer                        | What it does                                               |
|------------------------------|------------------------------------------------------------|
| `TraceLayer`                 | one line per request; outermost, so nothing goes unlogged  |
| `ResponseBodyTimeoutLayer`   | cuts a response body that stalls (`BODY_STALL_TIMEOUT`)    |
| `CompressionLayer`           | gzip or brotli, when the client asks for it                |
| `SetResponseHeaderLayer` ×4  | the security headers below                                 |
| `TimeoutLayer`               | 408 when no response within `RESPONSE_TIMEOUT` (30 s)      |
| `CatchPanicLayer`            | a panic becomes the logged 500 page, not a dropped socket   |

`/static` gets one more layer of its own, a `Cache-Control: no-cache` —
"revalidate", not "do not store", so `ServeDir`'s `ETag` turns the next
request into a cheap 304. It stays conservative because `main.css` has no
content hash in its name: a real `max-age` would serve a stale stylesheet
after a deploy.

### What the timeouts do and do not promise

Two layers, two different jobs, and it is worth being precise because the
names suggest more than they deliver:

- `TimeoutLayer` races a timer against the future that produces the
  `Response`. It bounds **the time to the first byte**, and nothing else —
  once a handler has returned, that layer is done and the body streams on.
- `ResponseBodyTimeoutLayer` bounds **how long the response body may go
  quiet** between two chunks, and the deadline resets on every chunk. A slow
  but progressing transfer still completes, so a large download over a thin
  link is not cut off.
- so there is **no hard ceiling on how long one request may last**, and
  nothing here bounds a slow *upload* either. Both are the job of rate
  limiting or of a reverse proxy, and this template has neither. Bounding the
  request body in the stack is possible — `tower-http` has
  `RequestBodyTimeoutLayer` — but it changes the body's type, which
  `Router::layer` refuses, so it needs a `MapRequestBodyLayer` to convert it
  back: two layers and a type trick for half a protection that belongs
  elsewhere. It was left out on purpose.

What *is* hard-bounded is the shutdown. `axum::serve`'s graceful shutdown
waits for every connection with no deadline of its own, so `src/main.rs`
adds `SHUTDOWN_GRACE` (20 s): once the signal arrives, connections get that
long to finish and the process then exits regardless.

The headers are `X-Content-Type-Options: nosniff`,
`Referrer-Policy: strict-origin-when-cross-origin`, `X-Frame-Options: DENY`,
and a `Content-Security-Policy` that matches what the templates actually load:
the stylesheet and the favicon from this origin, no inline style and no
script. Widen it in `app.rs` when you add a CDN or inline anything — an inline
`<script>` wants a nonce or a hash, never `'unsafe-inline'`.

`Strict-Transport-Security` is deliberately absent: it belongs to whatever
terminates TLS in front of the app, and this template serves plain HTTP in
development.

Every 5xx renders `templates/error.html` — a standalone page, with no layout
and no translation, so that it still works when i18n or the database is
exactly what broke. A caught panic goes through the same page: `app.rs` hands
`CatchPanicLayer` a custom responder rather than let `tower-http` answer with
its bare `text/plain` line, so a panic is not the one server error that looks
different. The cause goes to the logs and never to the visitor, which a test
in `src/error.rs` pins down.

The `lang` cookie is `HttpOnly` and `SameSite=Lax`. Only the server reads it,
so nothing is lost. Add `Secure` once you are behind TLS — it would stop the
cookie working over the plain HTTP used in development.

> [!WARNING]
> There is **no CSRF protection**, because there is no session to protect
> yet. Add one along with authentication — the `POST` routes of the notes
> example are otherwise reachable from any origin.
>
> When you do, mind that responses are compressed. A reply that holds **both**
> a secret and attacker-controlled text is a compression oracle (BREACH), and
> the notes example already echoes `form.title` back into its 422 page. Keep
> a CSRF token out of compressed HTML that reflects input, or give
> `CompressionLayer` a predicate that skips those responses.

## Tests

```sh
cargo test
```

The suite covers the shared code, not the `notes` example: language
negotiation (`src/i18n.rs`), the redirect guard of the language switcher
(`src/routes/lang.rs`), the menu helper and date formatting (`src/ctx.rs`),
the database URL handling (`src/db.rs`), the configuration fallbacks
(`src/config.rs`) and the 500 page (`src/error.rs`, which also checks that a
server error never leaks its cause to the visitor).

`src/app.rs` holds the router tests. They build the whole application —
every middleware, migrations applied — and answer requests through
`tower::ServiceExt::oneshot`, without a socket or a running server. Follow
them when you come to test your own pages:

```rust
let response = router("my-test")
    .await
    .oneshot(Request::builder().uri("/contact").body(Body::empty()).unwrap())
    .await
    .unwrap();
assert_eq!(response.status(), StatusCode::OK);
```

They run against a *named* in-memory SQLite database with `cache=shared`.
Plain `sqlite::memory:` would not do: every connection of the pool gets its
own empty database, so the migrations would be invisible to the next request.

> [!NOTE]
> The tests live in `#[cfg(test)]` modules inside `src/`, not in `tests/`.
> This crate is a binary, and an integration test in `tests/` cannot import a
> binary's modules — add a `src/lib.rs` if you would rather have them there.

## Logging

Logging goes through [`tracing`](https://docs.rs/tracing). `src/telemetry.rs`
installs one subscriber at startup, which prints to **stderr** and is driven
by the `RUST_LOG` environment variable (so it can be set in `.env`).

### What is logged out of the box

| Source        | What it gives you                                            |
|---------------|--------------------------------------------------------------|
| `tower_http`  | one line per HTTP request, with method, URI, status, latency |
| your own code | whatever you emit with `tracing::info!` and friends          |
| `AppError`    | the cause of every 5xx it returns                            |
| `i18n`        | a warning for each missing translation key                   |
| `sqlx`        | SQL statements with timing, from SeaORM's driver — see below |

### Setting the level with `RUST_LOG`

`RUST_LOG` is a comma-separated list of `target=level` directives, where a
bare level applies to everything not named. Levels, from most to least
verbose: `trace`, `debug`, `info`, `warn`, `error`, `off`.

When `RUST_LOG` is unset, `src/telemetry.rs` falls back to
`axum_template=debug,tower_http=info,info` — debug for this crate, info for
everything else. That default follows the crate name automatically, so
renaming the crate does not break it.

| Goal                        | Value                                                 |
|-----------------------------|-------------------------------------------------------|
| the default                 | `axum_template=debug,tower_http=info,info`            |
| see the SQL statements      | `axum_template=debug,tower_http=info,sqlx=debug,info` |
| quiet, production-like      | `warn`                                                |
| only this app, very verbose | `warn,axum_template=trace`                            |
| one module only             | `warn,axum_template::routes::notes=debug`             |
| drop the per-request lines  | `axum_template=debug,tower_http=warn,info`            |

Silencing `tower_http` also removes the `request{…}` span, since
`TraceLayer` is what opens it: your own events stay, but they no longer
say which request they belong to.

### Logging from your own code

Emit an event with a message and any number of fields. Prefer fields over
string interpolation — they stay machine-readable:

```rust
tracing::info!(id = note.id, "note created");
tracing::warn!(key, lang = "en", "missing translation");
tracing::error!(error = %err, "could not reach the mailer");  // % uses Display
```

To attach the arguments of a handler to everything it logs, annotate it.
`skip` leaves out what you do not want in the line — `Ctx` holds the whole
connection pool, and prints as a wall of noise:

```rust
#[tracing::instrument(skip(ctx))]
async fn remove(ctx: Ctx, Path(id): Path<i32>) -> AppResult<Redirect> {
```

That is what produces the `remove{id=6}` span above. See
`src/routes/notes.rs` for both in context.

### Changing the output format

The whole setup is the few lines of `src/telemetry.rs`. Useful tweaks:

- `NO_COLOR=1` drops the ANSI colour codes — worth setting when you pipe
  stderr to a file, since they are emitted even when it is not a terminal.
- `.with_target(false)`, `.compact()` or `.pretty()` on the `fmt()` builder
  change how much of each line you get.
- `.json()` switches to one JSON object per line, for a log collector. It
  needs the `json` feature: `tracing-subscriber = { version = "0.3",
  features = ["env-filter", "json"] }`.

## Going to production

```sh
cargo build --release
```

Ship the `target/release/axum-template` binary and the `static/` folder.
Templates and translations live inside the binary. Put the database on a
persistent volume, and set `APP_ADDR=0.0.0.0:3000`, `DATABASE_URL` and
`STATIC_DIR`.

`STATIC_DIR` has to point at a folder that exists, or the app refuses to
start — forgetting to ship `static/` is otherwise invisible until someone
notices the site has no styles. A variable set to an empty value falls back
to its default instead of being taken literally, so a blank `STATIC_DIR` in
a container's environment cannot turn into "serve the working directory".

## License

MIT — see [LICENSE](LICENSE).

