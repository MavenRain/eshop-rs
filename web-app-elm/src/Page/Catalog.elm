module Page.Catalog exposing (Model, Msg, init, update, view)

{-| Catalog browse.  Fetches the first page of catalog items from
`GET /api/catalog/items` on init and renders the
`PaginatedItemsResponse<CatalogItemResponse>` shape.
-}

import Api exposing (BaseUrl, Session)
import Html exposing (Html, div, h2, li, p, text, ul)
import Http
import Json.Decode as D


type alias Model =
    { items : LoadState (List CatalogItem)
    }


type LoadState a
    = Loading
    | Loaded a
    | Failed String


type alias CatalogItem =
    { id : Int
    , name : String
    , description : Maybe String
    , price : String
    , availableStock : Int
    }


type Msg
    = GotItems (Result Http.Error (List CatalogItem))


init : BaseUrl -> Session -> ( Model, Cmd Msg )
init base session =
    ( { items = Loading }
    , Api.get base session "/api/catalog/items" pageDecoder GotItems
    )


pageDecoder : D.Decoder (List CatalogItem)
pageDecoder =
    D.field "data" (D.list itemDecoder)


itemDecoder : D.Decoder CatalogItem
itemDecoder =
    D.map5 CatalogItem
        (D.field "id" D.int)
        (D.field "name" D.string)
        (D.field "description" (D.nullable D.string))
        (D.field "price" priceDecoder)
        (D.field "available_stock" D.int)


{-| `rust_decimal` serializes `Decimal` as a JSON string by default
(e.g. `"19.99"`).  We keep it as a `String` rather than parsing into
a `Float` so we don't lose precision; the renderer just prepends `$`.
-}
priceDecoder : D.Decoder String
priceDecoder =
    D.string


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotItems (Ok items) ->
            ( { model | items = Loaded items }, Cmd.none )

        GotItems (Err err) ->
            ( { model | items = Failed (errorToString err) }, Cmd.none )


errorToString : Http.Error -> String
errorToString err =
    case err of
        Http.BadUrl s ->
            "bad url: " ++ s

        Http.Timeout ->
            "request timed out"

        Http.NetworkError ->
            "network error"

        Http.BadStatus code ->
            "server returned " ++ String.fromInt code

        Http.BadBody body ->
            "decode failed: " ++ body


view : Model -> Html Msg
view model =
    div []
        [ h2 [] [ text "Catalog" ]
        , viewItems model.items
        ]


viewItems : LoadState (List CatalogItem) -> Html Msg
viewItems state =
    case state of
        Loading ->
            p [] [ text "Loading…" ]

        Failed reason ->
            p [] [ text ("Error: " ++ reason) ]

        Loaded items ->
            if List.isEmpty items then
                p [] [ text "No catalog items yet." ]

            else
                ul [] (List.map viewItem items)


viewItem : CatalogItem -> Html Msg
viewItem item =
    li []
        [ text
            (item.name
                ++ " — $"
                ++ item.price
                ++ " ("
                ++ String.fromInt item.availableStock
                ++ " in stock)"
            )
        ]
