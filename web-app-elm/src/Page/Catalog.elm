module Page.Catalog exposing (Model, Msg, init, update, view)

{-| Catalog browse + add-to-basket.

Fetches the first page of catalog items from `GET /api/catalog/items`
on init.  Each item carries an "Add to basket" button.  Adding is
modeled as a GET-then-PUT against `/api/basket`: we read the
existing basket items as opaque JSON (so the catalog page does not
have to know the basket-item schema), append the new item with
`id: null` (the server mints a fresh `BasketItemId`), and PUT the
combined list back.  The basket has replace semantics, so this is
the additive idiom.

Per-item add state is tracked in a `Dict Int AddStatus` keyed on
catalog id so multiple buttons can be in flight independently.

-}

import Api exposing (BaseUrl, Session)
import Dict exposing (Dict)
import Html exposing (Html, button, div, em, h2, li, p, span, text, ul)
import Html.Attributes as Attr
import Html.Events as Events
import Http
import Json.Decode as D
import Json.Encode as E


type alias Model =
    { items : LoadState (List CatalogItem)
    , addStates : Dict Int AddStatus
    , baseUrl : BaseUrl
    , session : Session
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
    , pictureFileName : Maybe String
    , availableStock : Int
    }


type AddStatus
    = Adding
    | AddedOk
    | AddFailed String


type Msg
    = GotItems (Result Http.Error (List CatalogItem))
    | AddClicked CatalogItem
    | GotBasket CatalogItem (Result Http.Error (List D.Value))
    | AddDone Int (Result Http.Error ())


init : BaseUrl -> Session -> ( Model, Cmd Msg )
init base session =
    ( { items = Loading
      , addStates = Dict.empty
      , baseUrl = base
      , session = session
      }
    , Api.get base session "/api/catalog/items" pageDecoder GotItems
    )


pageDecoder : D.Decoder (List CatalogItem)
pageDecoder =
    D.field "data" (D.list itemDecoder)


itemDecoder : D.Decoder CatalogItem
itemDecoder =
    D.map6 CatalogItem
        (D.field "id" D.int)
        (D.field "name" D.string)
        (D.field "description" (D.nullable D.string))
        (D.field "price" priceDecoder)
        (D.field "picture_file_name" (D.nullable D.string))
        (D.field "available_stock" D.int)


{-| `rust_decimal` serializes `Decimal` as a JSON string by default
(e.g. `"19.99"`).  We keep it as a `String` rather than parsing into
a `Float` so we don't lose precision; the renderer just prepends `$`
and the basket round-trip ships it back as a string.
-}
priceDecoder : D.Decoder String
priceDecoder =
    D.string


basketItemsDecoder : D.Decoder (List D.Value)
basketItemsDecoder =
    D.field "items" (D.list D.value)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotItems (Ok items) ->
            ( { model | items = Loaded items }, Cmd.none )

        GotItems (Err err) ->
            ( { model | items = Failed (errorToString err) }, Cmd.none )

        AddClicked item ->
            ( { model
                | addStates = Dict.insert item.id Adding model.addStates
              }
            , Api.get model.baseUrl
                model.session
                "/api/basket"
                basketItemsDecoder
                (GotBasket item)
            )

        GotBasket item (Ok existing) ->
            ( model
            , Api.put model.baseUrl
                model.session
                "/api/basket"
                (encodeMergedItems existing item)
                (D.succeed ())
                (AddDone item.id)
            )

        GotBasket item (Err err) ->
            ( { model
                | addStates =
                    Dict.insert item.id (AddFailed (errorToString err)) model.addStates
              }
            , Cmd.none
            )

        AddDone id (Ok _) ->
            ( { model | addStates = Dict.insert id AddedOk model.addStates }
            , Cmd.none
            )

        AddDone id (Err err) ->
            ( { model
                | addStates =
                    Dict.insert id (AddFailed (errorToString err)) model.addStates
              }
            , Cmd.none
            )


encodeMergedItems : List D.Value -> CatalogItem -> E.Value
encodeMergedItems existing item =
    E.object
        [ ( "items"
          , E.list identity (existing ++ [ encodeNewItem item ])
          )
        ]


{-| Encode a fresh catalog selection as a basket item.  `id` is
`null` so the server mints a `BasketItemId`; `quantity` defaults to
1; `old_unit_price` is null since this is the first time the item
enters this basket.
-}
encodeNewItem : CatalogItem -> E.Value
encodeNewItem item =
    E.object
        [ ( "id", E.null )
        , ( "product_id", E.int item.id )
        , ( "product_name", E.string item.name )
        , ( "unit_price", E.string item.price )
        , ( "old_unit_price", E.null )
        , ( "quantity", E.int 1 )
        , ( "picture_url", encodeNullable E.string item.pictureFileName )
        ]


encodeNullable : (a -> E.Value) -> Maybe a -> E.Value
encodeNullable enc value =
    case value of
        Just x ->
            enc x

        Nothing ->
            E.null


errorToString : Http.Error -> String
errorToString err =
    case err of
        Http.BadUrl s ->
            "bad url: " ++ s

        Http.Timeout ->
            "request timed out"

        Http.NetworkError ->
            "network error"

        Http.BadStatus 401 ->
            "please sign in"

        Http.BadStatus code ->
            "server returned " ++ String.fromInt code

        Http.BadBody body ->
            "decode failed: " ++ body


view : Model -> Html Msg
view model =
    div []
        [ h2 [] [ text "Catalog" ]
        , viewItems model.addStates model.items
        ]


viewItems : Dict Int AddStatus -> LoadState (List CatalogItem) -> Html Msg
viewItems addStates state =
    case state of
        Loading ->
            p [] [ text "Loading..." ]

        Failed reason ->
            p [] [ em [] [ text ("Error: " ++ reason) ] ]

        Loaded items ->
            if List.isEmpty items then
                p [] [ text "No catalog items yet." ]

            else
                ul [] (List.map (viewItem addStates) items)


viewItem : Dict Int AddStatus -> CatalogItem -> Html Msg
viewItem addStates item =
    li []
        [ text
            (item.name
                ++ ", $"
                ++ item.price
                ++ " ("
                ++ String.fromInt item.availableStock
                ++ " in stock) "
            )
        , viewAddButton addStates item
        ]


viewAddButton : Dict Int AddStatus -> CatalogItem -> Html Msg
viewAddButton addStates item =
    case Dict.get item.id addStates of
        Just Adding ->
            button [ Attr.disabled True ] [ text "Adding..." ]

        Just AddedOk ->
            button [ Attr.disabled True ] [ text "Added" ]

        Just (AddFailed reason) ->
            span []
                [ button
                    [ Events.onClick (AddClicked item) ]
                    [ text "Retry" ]
                , em [] [ text (", " ++ reason) ]
                ]

        Nothing ->
            button
                [ Events.onClick (AddClicked item) ]
                [ text "Add to basket" ]
