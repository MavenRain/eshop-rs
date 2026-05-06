module Page.Basket exposing (Model, Msg, init, update, view)

{-| Basket stub.  Renders a placeholder; follow-on slice fetches via
`GET /api/basket` (auth required) and renders the
`BasketResponse.items` array.
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
    p [] [ text "Basket goes here.  Sign in first; the next slice will GET /api/basket." ]
