## Common

app-name = Axum Template
nav-label = Main navigation
nav-home = Home
nav-notes = Notes
lang-switch = Language
footer-text = Powered by axum, askama, SeaORM and Fluent.

# chrono (strftime) format used by ctx.datetime().
# The `UTC` suffix is not decoration: timestamps are stored and printed in
# UTC, and the server never learns the visitor's timezone. Drop it only if you
# convert the time somewhere, for instance in the browser from the
# `<time datetime="…">` attribute, which carries the full RFC 3339 value.
format-datetime = %b %-d, %Y, %H:%M UTC

## Home

home-title = Welcome
home-intro = This project is a starting point: add your routes in src/routes/ and your pages in templates/.
home-note-count = { $count ->
    [one] You have { $count } note.
   *[other] You have { $count } notes.
}
home-cta = See notes

## Notes

notes-title = Notes
notes-new = New note
notes-empty = No notes yet.
notes-field-title = Title
notes-field-body = Content
notes-submit = Add
notes-delete = Delete
notes-error-title-required = A title is required.

## Errors

notfound-title = Page not found
notfound-text = The page you requested doesn't exist or has moved.
notfound-back = Back to home
notallowed-title = Method not allowed
notallowed-text = This address does not accept that kind of request.
