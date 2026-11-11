# axum-template

An opinionated Rust web application template using axum.

To do:

- [ ] Add an app file / struct will implement the services and utilities used by the project (db, i18n, context, etc).
- [ ] Configure the application's logging.
- [ ] Check for the static folder before starting the server.
- [ ] Add a context struct that will give access to the utilities (db, i81n, app conf ?) and is inserted in each
  request.
- [ ] Add a Router loaded by the application with a basic route.
    - [ ] The route must use an HTML template.
- [ ] Manage middlewares with a proper middleware structure that is used by the application with a first 404 catcher
  middleware.
- [ ] Add the following middlewares :
    - [ ] timout (408)
    - [ ] response body timeout
    - [ ] 500 catcher
    - [ ] compression
    - [ ] security headers
    - [ ] logging for the requests.
- [ ] Manage translations and give access to them trough the context.
- [ ] Add database management with entities :
    - [ ] follow the schema-first format of sea-orm.
    - [ ] the entities/ folder should be automatically generated (check that the prelude.rs is here).
    - [ ] give access to the database through the context.
    - [ ] Manage payload structs (DTO) to properly send and receive entities without leaking sensitive data.

| Role           | Tool                                        |
|----------------|---------------------------------------------|
| Server / API   | axum 0.8                                    |
| HTML templates | askama 0.16 (+ askama_web)                  |
| Translation    | Fluent (fluent-templates)                   |
| Database       | SQLite with SeaORM 2                        |
| Logs           | tracing + tower-http                        |
| Styles         | SCSS compiled by Dart Sass on `cargo build` |

## Requirements

- [Dart Sass](https://sass-lang.com/install), in `PATH` (`sass --version`) or
  pointed at by `SASS_BIN`. It is **required**: the build fails without it,
  for `cargo check` and `cargo test` too — see [SCSS](docs/scss.md)
- Optional: `cargo install sea-orm-cli@^2` (generate migrations and entities)
- Optional: `cargo install watchexec-cli` (automatic reload)

## Getting started

```sh
cp .env.example .env
cargo run
```

Then open <http://127.0.0.1:3000>.

Rename the project by changing `name` in `Cargo.toml` and in the `RUST_LOG` variable from the `.env`.
The default log filter follows the crate name automatically.

To rebuild and restart on every change:

```sh
watchexec -r -e rs,html,ftl,scss,toml -- cargo run
```

## License

MIT — see [LICENSE](LICENSE).
