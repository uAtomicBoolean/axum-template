# axum-template

An opinionated Rust web application template using axum.

> [!NOTE]
> The template comes with a notes example in `src/routes/notes.rs` — a full CRUD (form, validation,
> redirect) — and a JSON route in `src/routes/api.rs`. Both are there to be read and replaced, not
> shipped as they are.

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
- [Dart Sass](https://sass-lang.com/install), in `PATH` (`sass --version`) or
  pointed at by `SASS_BIN`. It is **required**: the build fails without it,
  for `cargo check` and `cargo test` too — see [SCSS](docs/scss.md)
- Optional: `cargo install sea-orm-cli@^2` (generate migrations and entities)
- Optional: `cargo install watchexec-cli` (automatic reload)

## Getting started

```sh
cp .env.example .env     # optional here, required by the migration CLI
cargo run
```

Then open <http://127.0.0.1:3000>. On the first run, the `data/app.db`
database is created and the migrations are applied.

Rename the project by changing `name` in `Cargo.toml`. The default log filter
follows the crate name automatically.

Every variable has a default in `src/config.rs`, so the application runs
without a `.env`. The migration CLI is the one exception: it reads
`DATABASE_URL` from the environment and panics when it is missing.

To rebuild and restart on every change:

```sh
watchexec -r -e rs,html,ftl,scss,toml -- cargo run
```

## Project structure

```
├── build.rs                  compiles styles/ → static/css/main.css (Dart Sass)
├── docs/                     the documentation below
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

The *why* of each choice is in the doc comments, next to the code. The pages
below cover the rest.

## Documentation

| Page | What it covers |
|---|---|
| [Adding to the template](docs/extending.md) | the `Ctx` context, adding a page, adding a table |
| [Translations](docs/i18n.md) | Fluent, language negotiation, plurals, dates |
| [SCSS](docs/scss.md) | Dart Sass, `SASS_BIN`, working on the styles |
| [Middleware and security](docs/middleware.md) | the layer stack, headers, what the timeouts do not promise, **what is missing** |
| [Tests](docs/testing.md) | what is covered, how to test a page, lints |
| [Logging](docs/logging.md) | `RUST_LOG`, what is logged, output formats |

## Going to production

```sh
cargo build --release
```

Ship the `target/release/axum-template` binary and the `static/` folder.
Templates and translations live inside the binary. Put the database on a
persistent volume, and set `APP_ADDR=0.0.0.0:3000`, `DATABASE_URL` and
`STATIC_DIR`.

Three things to know before you do:

- `STATIC_DIR` has to point at a folder that exists, or the app refuses to
  start — forgetting to ship `static/` is otherwise invisible until someone
  notices the site has no styles. A variable set to an empty value falls back
  to its default, so a blank one cannot turn into "serve the working
  directory".
- there is **no CSRF protection** and **no rate limiting**, and the timeouts
  promise less than their names suggest. Read
  [Middleware and security](docs/middleware.md) first.
- build what you ship with `cargo build --release` alone, not
  `cargo build --workspace`, which pulls the migration CLI's dependencies
  back into the binary.

## License

MIT — see [LICENSE](LICENSE).
