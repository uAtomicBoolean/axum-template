# Middleware and security

[← README](../README.md)

This page is the **map**. The reasoning behind each choice lives in the doc
comments of [`src/app.rs`](../src/app.rs), next to the line it explains, where
it cannot drift away from the code — read them when you change a layer.

## The stack

`src/app.rs` wraps every route, static files included, in one
`ServiceBuilder`, which reads in the order requests travel: first layer
outermost, last one against the handler.

| Layer                       | What it does                                              |
|-----------------------------|-----------------------------------------------------------|
| `TraceLayer`                | one line per request; outermost, so nothing goes unlogged |
| `ResponseBodyTimeoutLayer`  | cuts a response body that stalls (`BODY_STALL_TIMEOUT`)   |
| `CompressionLayer`          | gzip or brotli, when the client asks for it               |
| `SetResponseHeaderLayer` ×4 | the security headers below                                |
| `TimeoutLayer`              | 408 when no response within `RESPONSE_TIMEOUT` (30 s)     |
| `CatchPanicLayer`           | a panic becomes the logged 500 page, not a dropped socket |

`/static` gets one more of its own: `Cache-Control: no-cache`.

## Security headers

| Header | Value |
|---|---|
| `X-Content-Type-Options` | `nosniff` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |
| `X-Frame-Options` | `DENY` |
| `Content-Security-Policy` | same-origin stylesheet and favicon, no inline style, no script |

`Strict-Transport-Security` is deliberately absent: it belongs to whatever
terminates TLS in front of the app.

The `lang` cookie is `HttpOnly` and `SameSite=Lax`; add `Secure` once you are
behind TLS.

## What the timeouts do not promise

The names suggest more than they deliver. `TimeoutLayer` bounds the time to
the **first byte** only; `ResponseBodyTimeoutLayer` bounds how long a response
body may go **quiet**, resetting on each chunk. So there is **no ceiling on
how long a request may last**, and nothing bounds a slow *upload* at all —
both belong to rate limiting or to a reverse proxy, which this template does
not have.

What *is* hard-bounded is the shutdown: `axum::serve` waits for every
connection with no deadline of its own, so `src/main.rs` adds
`SHUTDOWN_GRACE` (20 s) and exits regardless once it runs out.

## Errors

A 404 gets the real translated page, from the router's fallback or from
`AppError::NotFound` alike; a 405 has its own. Every 5xx, caught panics
included, renders `templates/error.html` — standalone, untranslated, so it
survives i18n or the database being what broke — and the cause goes to the
logs, never to the visitor.

## What is missing

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

No rate limiting either, and no `/health` route.
