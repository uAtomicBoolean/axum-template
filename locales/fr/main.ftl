## Common

app-name = Axum Template
nav-label = Navigation principale
nav-home = Accueil
nav-notes = Notes
lang-switch = Langue
footer-text = Propulsé par axum, askama, SeaORM et Fluent.

# chrono (strftime) format used by ctx.datetime().
# The `UTC` suffix is not decoration: timestamps are stored and printed in
# UTC, and the server never learns the visitor's timezone. Drop it only if you
# convert the time somewhere, for instance in the browser from the
# `<time datetime="…">` attribute, which carries the full RFC 3339 value.
format-datetime = %d/%m/%Y à %H:%M UTC

## Home

home-title = Bienvenue
home-intro = Ce projet est un point de départ : ajoutez vos routes dans src/routes/ et vos pages dans templates/.
home-note-count = { $count ->
    [one] Vous avez { $count } note.
   *[other] Vous avez { $count } notes.
}
home-cta = Voir les notes

## Notes

notes-title = Notes
notes-new = Nouvelle note
notes-empty = Aucune note pour l'instant.
notes-field-title = Titre
notes-field-body = Contenu
notes-submit = Ajouter
notes-delete = Supprimer
notes-error-title-required = Le titre est obligatoire.

## Errors

notfound-title = Page introuvable
notfound-text = La page demandée n'existe pas ou a été déplacée.
notfound-back = Retour à l'accueil
notallowed-title = Méthode non autorisée
notallowed-text = Cette adresse n'accepte pas ce type de requête.
