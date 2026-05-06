module Page.Catalog exposing (Model, Msg, init, update, view)

{-| Catalog browse stub.  Renders a placeholder list; the follow-on
slice fetches items via `GET /api/catalog/items` and decodes the
`PaginatedItemsResponse<CatalogItemResponse>` shape into the model.
-}

import Html exposing (Html, p, text)


type alias Model =
    {}


type Msg
    = NoOp


init : ( Model, Cmd Msg )
init =
    ( {}, Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update _ model =
    ( model, Cmd.none )


view : Model -> Html Msg
view _ =
    p [] [ text "Catalog browse goes here.  Wire up GET /api/catalog/items in the next slice." ]
