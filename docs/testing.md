# Tests

[← README](../README.md)

```sh
cargo test
```

The suite covers the shared code, not the `notes` example:

| File | What is tested |
|---|---|
| `src/i18n.rs` | language negotiation, `q=` weights, cookie priority, locale key parity |
| `src/routes/lang.rs` | the redirect guard of the language switcher |
| `src/ctx.rs` | the menu helper, date formatting and its UTC suffix |
| `src/db.rs` | database URL handling and parent-folder creation |
| `src/config.rs` | the configuration fallbacks and the redacted `Debug` |
| `src/error.rs` | the 500 page, and that it never leaks its cause |
| `src/app.rs` | the router, end to end |

## Router tests

`src/app.rs` builds the whole application — every middleware, migrations
applied — and answers requests through `tower::ServiceExt::oneshot`, without
a socket or a running server. Follow them when you come to test your own
pages:

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

`src/routes/mod.rs` registers one extra route under `#[cfg(test)]`,
`/panic-on-purpose`, which is how the panic test reaches `CatchPanicLayer`
through the real stack. It does not exist in a built server.

> [!NOTE]
> The tests live in `#[cfg(test)]` modules inside `src/`, not in `tests/`.
> This crate is a binary, and an integration test in `tests/` cannot import a
> binary's modules — add a `src/lib.rs` if you would rather have them there.

## Lints

`Cargo.toml` denies `clippy::all` and forbids `unsafe_code`, so a violation
fails `cargo clippy` with no flag to remember. Plain rustc warnings — unused
variables, dead code — are still only warnings.
