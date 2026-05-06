# web-app-elm

Elm SPA frontend for the `eshop-rs` apphost.  Mirrors upstream
`dotnet/eShop`'s WebApp at the URL surface; renders against the
HTTP API exposed by `eshop-apphost`.

## Layout

```
src/
  Main.elm       application entry; routes + page composition
  Api.elm        talk-to-apphost helpers (BaseUrl, Session, get, post)
  Route.elm      URL parser + Route enum
  Page/
    Home.elm
    Login.elm
    Catalog.elm
    Basket.elm
```

Each `Page/*.elm` exposes the standard `Model`, `Msg`, `init`,
`update`, `view` quintet; `Main.elm` wraps page messages with
`HomeMsg` / `LoginMsg` / `CatalogMsg` / `BasketMsg` so dispatch
stays exhaustive and the compiler enforces it.

## Build

```bash
cd web-app-elm
elm make src/Main.elm --output=dist/main.js
```

Then serve `index.html` with any static HTTP server.  The page reads
`window.ESHOP_API_BASE` (set in a `<script>` if you want to point at
something other than `http://127.0.0.1:8080`) and feeds it into the
Elm app as a flag.

## Status

This is the scaffolding slice (5A on the eshop-rs punchlist).  Pages
render placeholder content; the apphost endpoints are not yet
wired.  Follow-on slices add:

- 5B: catalog browse — `GET /api/catalog/items`, render the paginated
  response.
- 5C: identity — `POST /api/identity/login`, store the bearer token
  in the model, surface it via `Api.Session`.
- 5D: basket — `GET /api/basket`, item add/remove, checkout.
- 5E: webhooks management UI for the auth'd user.
