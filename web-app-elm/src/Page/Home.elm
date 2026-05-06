module Page.Home exposing (Model, Msg, init, update, view)

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
    p [] [ text "Welcome to the eShop Rust port.  Browse the catalog or sign in to manage a basket." ]
