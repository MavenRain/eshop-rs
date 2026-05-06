module Page.Login exposing (Model, Msg, consumeToken, init, update, view)

{-| Login form.  Renders the email + password inputs; on submit
fires `POST /api/identity/login` and surfaces the resulting bearer
token to `Main` via [`consumeToken`](#consumeToken).
-}

import Api exposing (BaseUrl, Session)
import Html exposing (Html, button, div, form, input, label, p, text)
import Html.Attributes exposing (disabled, placeholder, type_, value)
import Html.Events exposing (onInput, onSubmit)
import Http
import Json.Decode as D
import Json.Encode as E


type alias Model =
    { email : String
    , password : String
    , status : Status
    , issuedToken : Maybe String
    }


type Status
    = Idle
    | Submitting
    | Errored String


type Msg
    = EmailChanged String
    | PasswordChanged String
    | Submitted
    | GotResponse (Result Http.Error String)


init : ( Model, Cmd Msg )
init =
    ( { email = "", password = "", status = Idle, issuedToken = Nothing }
    , Cmd.none
    )


update : BaseUrl -> Session -> Msg -> Model -> ( Model, Cmd Msg )
update base session msg model =
    case msg of
        EmailChanged next ->
            ( { model | email = next }, Cmd.none )

        PasswordChanged next ->
            ( { model | password = next }, Cmd.none )

        Submitted ->
            ( { model | status = Submitting }
            , Api.post base session "/api/identity/login" (encodeRequest model) tokenDecoder GotResponse
            )

        GotResponse (Ok token) ->
            ( { model | status = Idle, issuedToken = Just token, password = "" }
            , Cmd.none
            )

        GotResponse (Err err) ->
            ( { model | status = Errored (errorToString err) }
            , Cmd.none
            )


encodeRequest : Model -> E.Value
encodeRequest model =
    E.object
        [ ( "email", E.string model.email )
        , ( "password", E.string model.password )
        ]


tokenDecoder : D.Decoder String
tokenDecoder =
    D.field "access_token" D.string


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
            "invalid email or password"

        Http.BadStatus code ->
            "server returned " ++ String.fromInt code

        Http.BadBody body ->
            "decode failed: " ++ body


{-| One-shot accessor.  Returns the model with `issuedToken` zeroed
plus the freshly-issued token, if any.  `Main` calls this after every
LoginMsg dispatch and lifts the token into its `Session`.

The "consume" semantics mean a re-render of the Login page won't
re-issue the token, even though the page model continues to live.
-}
consumeToken : Model -> ( Model, Maybe String )
consumeToken model =
    case model.issuedToken of
        Just token ->
            ( { model | issuedToken = Nothing }, Just token )

        Nothing ->
            ( model, Nothing )


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
            [ button
                [ type_ "submit", disabled (model.status == Submitting) ]
                [ text (submitLabel model.status) ]
            ]
        , viewStatus model.status
        ]


submitLabel : Status -> String
submitLabel status =
    case status of
        Idle ->
            "Sign in"

        Submitting ->
            "Signing in…"

        Errored _ ->
            "Try again"


viewStatus : Status -> Html Msg
viewStatus status =
    case status of
        Idle ->
            text ""

        Submitting ->
            text ""

        Errored reason ->
            p [] [ text reason ]
