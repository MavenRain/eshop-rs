module Page.Basket exposing (Model, Msg, init, update, view)

{-| Authenticated basket flow.

On entry: `GET /api/basket`.  On "Remove": `PUT /api/basket` with
the remaining items (basket has replace semantics).  On "Checkout":
`POST /api/basket/checkout`, which deletes the basket and returns
the pre-deletion snapshot — we render that snapshot under a
"thank you" headline.
-}

import Api exposing (BaseUrl, Session)
import Html exposing (Html, a, button, div, h2, li, p, text, ul)
import Html.Events exposing (onClick)
import Http
import Json.Decode as D
import Json.Encode as E
import Route


type alias Model =
    { state : State
    , baseUrl : BaseUrl
    , session : Session
    }


type State
    = Loading
    | Active Basket
    | CheckedOut Basket
    | Failed String


type alias Basket =
    { items : List BasketItem }


type alias BasketItem =
    { id : String
    , productId : Int
    , productName : String
    , unitPrice : String
    , oldUnitPrice : Maybe String
    , quantity : Int
    , pictureUrl : Maybe String
    }


type Msg
    = GotBasket (Result Http.Error Basket)
    | RemoveClicked String
    | ItemsReplaced (Result Http.Error Basket)
    | CheckoutClicked
    | CheckoutDone (Result Http.Error Basket)


init : BaseUrl -> Session -> ( Model, Cmd Msg )
init base session =
    ( { state = Loading, baseUrl = base, session = session }
    , Api.get base session "/api/basket" basketDecoder GotBasket
    )


basketDecoder : D.Decoder Basket
basketDecoder =
    D.map Basket (D.field "items" (D.list itemDecoder))


itemDecoder : D.Decoder BasketItem
itemDecoder =
    D.map7 BasketItem
        (D.field "id" D.string)
        (D.field "product_id" D.int)
        (D.field "product_name" D.string)
        (D.field "unit_price" D.string)
        (D.field "old_unit_price" (D.nullable D.string))
        (D.field "quantity" D.int)
        (D.field "picture_url" (D.nullable D.string))


encodeUpdateRequest : List BasketItem -> E.Value
encodeUpdateRequest items =
    E.object [ ( "items", E.list encodeItem items ) ]


encodeItem : BasketItem -> E.Value
encodeItem item =
    E.object
        [ ( "id", E.string item.id )
        , ( "product_id", E.int item.productId )
        , ( "product_name", E.string item.productName )
        , ( "unit_price", E.string item.unitPrice )
        , ( "old_unit_price", encodeNullable E.string item.oldUnitPrice )
        , ( "quantity", E.int item.quantity )
        , ( "picture_url", encodeNullable E.string item.pictureUrl )
        ]


encodeNullable : (a -> E.Value) -> Maybe a -> E.Value
encodeNullable enc value =
    case value of
        Just x ->
            enc x

        Nothing ->
            E.null


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotBasket (Ok basket) ->
            ( { model | state = Active basket }, Cmd.none )

        GotBasket (Err err) ->
            ( { model | state = Failed (errorToString err) }, Cmd.none )

        RemoveClicked itemId ->
            removeItem itemId model

        ItemsReplaced (Ok basket) ->
            ( { model | state = Active basket }, Cmd.none )

        ItemsReplaced (Err err) ->
            ( { model | state = Failed (errorToString err) }, Cmd.none )

        CheckoutClicked ->
            ( model
            , Api.post model.baseUrl
                model.session
                "/api/basket/checkout"
                (E.object [])
                basketDecoder
                CheckoutDone
            )

        CheckoutDone (Ok basket) ->
            ( { model | state = CheckedOut basket }, Cmd.none )

        CheckoutDone (Err err) ->
            ( { model | state = Failed (errorToString err) }, Cmd.none )


removeItem : String -> Model -> ( Model, Cmd Msg )
removeItem itemId model =
    case model.state of
        Active basket ->
            let
                remaining =
                    List.filter (\item -> item.id /= itemId) basket.items
            in
            -- The PUT body's `items` list contains every survivor;
            -- basket has replace semantics.
            ( model
            , Api.put model.baseUrl
                model.session
                "/api/basket"
                (encodeUpdateRequest remaining)
                basketDecoder
                ItemsReplaced
            )

        Loading ->
            ( model, Cmd.none )

        CheckedOut _ ->
            ( model, Cmd.none )

        Failed _ ->
            ( model, Cmd.none )


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
        [ h2 [] [ text "Basket" ]
        , viewState model.state
        ]


viewState : State -> Html Msg
viewState state =
    case state of
        Loading ->
            p [] [ text "Loading…" ]

        Failed reason ->
            p [] [ text ("Error: " ++ reason) ]

        Active basket ->
            viewActive basket

        CheckedOut basket ->
            viewCheckedOut basket


viewActive : Basket -> Html Msg
viewActive basket =
    if List.isEmpty basket.items then
        p [] [ text "Your basket is empty." ]

    else
        div []
            [ ul [] (List.map viewActiveItem basket.items)
            , button [ onClick CheckoutClicked ] [ text "Checkout" ]
            ]


viewActiveItem : BasketItem -> Html Msg
viewActiveItem item =
    li []
        [ text (itemSummary item)
        , text " "
        , button [ onClick (RemoveClicked item.id) ] [ text "Remove" ]
        ]


viewCheckedOut : Basket -> Html Msg
viewCheckedOut basket =
    div []
        [ p [] [ text "Order placed.  Thanks!" ]
        , ul [] (List.map viewSnapshotItem basket.items)
        , p [] [ a [ Route.href Route.Orders ] [ text "View your orders" ] ]
        ]


viewSnapshotItem : BasketItem -> Html Msg
viewSnapshotItem item =
    li [] [ text (itemSummary item) ]


itemSummary : BasketItem -> String
itemSummary item =
    item.productName
        ++ " — $"
        ++ item.unitPrice
        ++ " x "
        ++ String.fromInt item.quantity
