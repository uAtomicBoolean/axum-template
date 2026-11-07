# SCSS

[← README](../README.md)

> [!NOTE]
> The template requires Dart Sass to be installed on the computer.
> This is a deliberate choice.

`build.rs` runs `sass` on every build where `styles/` has changed (CSS is
compressed in `--release`). There is no way to skip it: without Dart Sass the
build fails rather than quietly producing a server that answers every page
unstyled. An SCSS syntax error fails the build too.

Because the build script runs for `cargo check`, `cargo test` and
`cargo clippy` as well, Dart Sass is needed for all of them — including in CI,
where it has to be installed in the image.

## Finding Dart Sass

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

> [!WARNING]
> On Windows, write the path in `.env` with forward slashes
> (`SASS_BIN=E:/bin/sass`) or in single quotes (`SASS_BIN='E:\bin\sass'`). A
> bare backslash is an escape character there, so `E:\bin\sass` makes the
> **whole file** unreadable — `DATABASE_URL` and `APP_ADDR` included. The
> build warns when that happens.

## Working on the styles

To iterate on the CSS without recompiling the Rust, run

```sh
sass --watch --load-path=styles styles/main.scss:static/css/main.css
```

alongside the server, then reload the page.

`styles/` is split into `_tokens.scss` (Sass variables, spacing scale,
breakpoint mixin — emits no CSS), `_base.scss` (colour variables for both
themes, element defaults), `_layout.scss` and `_components.scss`. Colours are
CSS custom properties, so dark mode is one `@media (prefers-color-scheme)`
block and nothing else.

If you delete `static/css/main.css` by hand, touch a file in `styles/` so
that it gets regenerated: `build.rs` only re-runs when one of its tracked
inputs changes, and its own output is not one of them.
