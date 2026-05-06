module Page.Orders exposing (Model, Msg, init, update, view)

{-| Orders listing.

Fetches `GET /api/orders` on init and renders one section per
order with its line items.  Auth-required; the bearer header
attaches automatically through [`Api`](Api).

The address fields on the response are not rendered: until the
basket-checkout saga grows shipping-address capture, every order
this app produces carries sentinel `"TBD"` values for them, so
showing them would be noise.

-}

import Api exposing (BaseUrl, Session)
import Html exposing (Html, div, em, h2, h3, li, p, strong, text, ul)
import Http
import Json.Decode as D


type alias Model =
    { state : State
    }


type State
    = Loading
    | Loaded (List Order)
    | Failed String


type alias Order =
    { id : String
    , orderDate : String
    , orderStatus : String
    , description : String
    , items : List OrderItem
    }


type alias OrderItem =
    { itemId : String
    , productId : Int
    , productName : String
    , pictureUrl : String
    , unitPrice : String
    , discount : String
    , units : Int
    }


type Msg
    = GotOrders (Result Http.Error (List Order))


init : BaseUrl -> Session -> ( Model, Cmd Msg )
init baseUrl session =
    ( { state = Loading }
    , Api.get baseUrl session "/api/orders" ordersDecoder GotOrders
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotOrders (Ok orders) ->
            ( { model | state = Loaded orders }, Cmd.none )

        GotOrders (Err err) ->
            ( { model | state = Failed (errorToString err) }, Cmd.none )


ordersDecoder : D.Decoder (List Order)
ordersDecoder =
    D.list orderDecoder


orderDecoder : D.Decoder Order
orderDecoder =
    D.map5 Order
        (D.field "order_id" D.string)
        (D.field "order_date" D.string)
        (D.field "order_status" D.string)
        (D.field "description" D.string)
        (D.field "items" (D.list itemDecoder))


itemDecoder : D.Decoder OrderItem
itemDecoder =
    D.map7 OrderItem
        (D.field "item_id" D.string)
        (D.field "product_id" D.int)
        (D.field "product_name" D.string)
        (D.field "picture_url" D.string)
        (D.field "unit_price" D.string)
        (D.field "discount" D.string)
        (D.field "units" D.int)


errorToString : Http.Error -> String
errorToString err =
    case err of
        Http.BadUrl detail ->
            "bad url: " ++ detail

        Http.Timeout ->
            "request timed out"

        Http.NetworkError ->
            "network error"

        Http.BadStatus 401 ->
            "please sign in"

        Http.BadStatus code ->
            "server returned " ++ String.fromInt code

        Http.BadBody detail ->
            "could not decode response: " ++ detail


view : Model -> Html Msg
view model =
    div []
        [ h2 [] [ text "Your orders" ]
        , viewState model.state
        ]


viewState : State -> Html Msg
viewState state =
    case state of
        Loading ->
            p [] [ text "Loading orders..." ]

        Failed reason ->
            p [] [ em [] [ text ("Could not load orders: " ++ reason) ] ]

        Loaded [] ->
            p [] [ em [] [ text "You have no orders yet." ] ]

        Loaded orders ->
            div [] (List.map viewOrder orders)


viewOrder : Order -> Html Msg
viewOrder order =
    div []
        [ h3 []
            [ text "Order "
            , text (truncateId order.id)
            ]
        , p []
            [ strong [] [ text "Status: " ]
            , text order.orderStatus
            , text " (placed "
            , text order.orderDate
            , text ")"
            ]
        , viewDescription order.description
        , ul [] (List.map viewItem order.items)
        ]


viewDescription : String -> Html msg
viewDescription description =
    if String.isEmpty description then
        text ""

    else
        p [] [ em [] [ text description ] ]


viewItem : OrderItem -> Html Msg
viewItem item =
    li []
        [ text (String.fromInt item.units ++ "x ")
        , strong [] [ text item.productName ]
        , text (" @ $" ++ item.unitPrice)
        , viewDiscount item.discount
        ]


viewDiscount : String -> Html msg
viewDiscount discount =
    if discount == "0" || discount == "0.0" || discount == "0.00" then
        text ""

    else
        text (" (less $" ++ discount ++ " discount)")


truncateId : String -> String
truncateId id =
    String.left 8 id
