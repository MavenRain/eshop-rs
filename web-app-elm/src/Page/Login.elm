module Page.Login exposing (Model, Msg, init, update, view)

{-| Login form scaffold.  Renders the input fields and accepts
edits; submission is wired in a follow-on slice that will call
`POST /api/identity/login` and surface the bearer token to `Main`.
-}

import Html exposing (Html, button, div, form, input, label, text)
import Html.Attributes exposing (placeholder, type_, value)
import Html.Events exposing (onInput, onSubmit)


type alias Model =
    { email : String
    , password : String
    , status : Status
    }


type Status
    = Idle
    | Submitting


type Msg
    = EmailChanged String
    | PasswordChanged String
    | Submitted


init : ( Model, Cmd Msg )
init =
    ( { email = "", password = "", status = Idle }, Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        EmailChanged value_ ->
            ( { model | email = value_ }, Cmd.none )

        PasswordChanged value_ ->
            ( { model | password = value_ }, Cmd.none )

        Submitted ->
            -- Wire to Api.post once the slice that adds login lands.
            ( { model | status = Submitting }, Cmd.none )


view : Model -> Html Msg
view model =
    form [ onSubmit Submitted ]
        [ div []
            [ label [] [ text "Email" ]
            , input
                [ type_ "email"
                , placeholder "alice@example.test"
                , value model.email
                , onInput EmailChanged
                ]
                []
            ]
        , div []
            [ label [] [ text "Password" ]
            , input
                [ type_ "password"
                , value model.password
                , onInput PasswordChanged
                ]
                []
            ]
        , div []
            [ button [ type_ "submit" ]
                [ text
                    (case model.status of
                        Idle ->
                            "Sign in"

                        Submitting ->
                            "Signing in…"
                    )
                ]
            ]
        ]
