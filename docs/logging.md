# Logging

[← README](../README.md)

Logging goes through [`tracing`](https://docs.rs/tracing). `src/telemetry.rs`
installs one subscriber at startup, which prints to **stderr** and is driven
by the `RUST_LOG` environment variable (so it can be set in `.env`).

## What is logged out of the box

| Source        | What it gives you                                            |
|---------------|--------------------------------------------------------------|
| `tower_http`  | one line per HTTP request, with method, URI, status, latency |
| your own code | whatever you emit with `tracing::info!` and friends          |
| `AppError`    | the cause of every 5xx it returns                            |
| `i18n`        | a warning for each missing translation key                   |
| `sqlx`        | SQL statements with timing, from SeaORM's driver — see below |

## Setting the level with `RUST_LOG`

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

> [!NOTE]
> An unreadable `RUST_LOG` is ignored without a word: the subscriber falls
> back to the default filter. A typo in a production `RUST_LOG=warn` therefore
> gives you the verbose development filter, silently.

The per-request line carries the full URI, query string included. Keep tokens
and personal data out of query strings, or they end up in the logs.

## Logging from your own code

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

That is what produces the `remove{id=6}` span. See `src/routes/notes.rs` for
both in context.

`Config` implements `Debug` by hand so that `database_url` prints as
`<redacted>`: it is harmless for the SQLite URL shipped here, but it is the
one value you are going to change, and a PostgreSQL URL carries
`user:password@host`. Do the same for any secret you add to the
configuration — deriving `Debug` on it is enough to put it in a log.

## Changing the output format

The whole setup is the few lines of `src/telemetry.rs`. Useful tweaks:

- `NO_COLOR=1` drops the ANSI colour codes — worth setting when you pipe
  stderr to a file, since they are emitted even when it is not a terminal.
- `.with_target(false)`, `.compact()` or `.pretty()` on the `fmt()` builder
  change how much of each line you get.
- `.json()` switches to one JSON object per line, for a log collector. It
  needs the `json` feature: `tracing-subscriber = { version = "0.3",
  features = ["env-filter", "json"] }`.
