module Page.Register exposing (Model, Msg, init, update, view)

{-| Account registration.

Posts a `{ email, name, password }` body to `POST /api/identity/register`.
Backend mints a `User` and returns `{ id }` on success; the SPA does
not auto-login because the register endpoint does not issue a token.
On success we render a "your account is ready" message with a link
to `/login`; the user signs in there to receive the bearer token.

State machine:

  - `Idle` — form is fillable.
  - `Submitting` — request in flight; submit button disabled.
  - `Errored reason` — request failed; reason rendered, form
    remains fillable.
  - `Succeeded` — registration complete; the form is hidden in
    favor of a "go sign in" call-to-action.

-}

import Api exposing (BaseUrl, Session)
import Html exposing (Html, a, button, div, form, h2, input, label, p, text)
import Html.Attributes exposing (disabled, placeholder, type_, value)
import Html.Events exposing (onInput, onSubmit)
import Http
import Json.Decode as D
import Json.Encode as E
import Route


type alias Model =
    { email : String
    , name : String
    , password : String
    , status : Status
    }


type Status
    = Idle
    | Submitting
    | Errored String
    | Succeeded


type Msg
    = EmailChanged String
    | NameChanged String
    | PasswordChanged String
    | Submitted
    | GotResponse (Result Http.Error String)


init : ( Model, Cmd Msg )
init =
    ( { email = "", name = "", password = "", status = Idle }
    , Cmd.none
    )


update : BaseUrl -> Session -> Msg -> Model -> ( Model, Cmd Msg )
update base session msg model =
    case msg of
        EmailChanged next ->
            ( { model | email = next, status = stripError model.status }, Cmd.none )

        NameChanged next ->
            ( { model | name = next, status = stripError model.status }, Cmd.none )

        PasswordChanged next ->
            ( { model | password = next, status = stripError model.status }, Cmd.none )

        Submitted ->
            if canSubmit model then
                ( { model | status = Submitting }
                , Api.post base
                    session
                    "/api/identity/register"
                    (encodeRequest model)
                    idDecoder
                    GotResponse
                )

            else
                ( model, Cmd.none )

        GotResponse (Ok _) ->
            ( { model | status = Succeeded, password = "" }, Cmd.none )

        GotResponse (Err err) ->
            ( { model | status = Errored (errorToString err) }, Cmd.none )


canSubmit : Model -> Bool
canSubmit model =
    let
        nonEmpty =
            String.trim >> String.isEmpty >> not
    in
    nonEmpty model.email
        && nonEmpty model.name
        && nonEmpty model.password
        && model.status
        /= Submitting


stripError : Status -> Status
stripError status =
    case status of
        Errored _ ->
            Idle

        other ->
            other


encodeRequest : Model -> E.Value
encodeRequest model =
    E.object
        [ ( "email", E.string model.email )
        , ( "name", E.string model.name )
        , ( "password", E.string model.password )
        ]


idDecoder : D.Decoder String
idDecoder =
    D.field "id" D.string


errorToString : Http.Error -> String
errorToString err =
    case err of
        Http.BadUrl s ->
            "bad url: " ++ s

        Http.Timeout ->
            "request timed out"

        Http.NetworkError ->
            "network error"

        Http.BadStatus 409 ->
            "an account with that email already exists"

        Http.BadStatus 400 ->
            "please check the email is well-formed and no field is blank"

        Http.BadStatus code ->
            "server returned " ++ String.fromInt code

        Http.BadBody body ->
            "decode failed: " ++ body


view : Model -> Html Msg
view model =
    div []
        [ h2 [] [ text "Create an account" ]
        , viewBody model
        ]


viewBody : Model -> Html Msg
viewBody model =
    case model.status of
        Succeeded ->
            viewSuccess

        _ ->
            viewForm model


viewSuccess : Html Msg
viewSuccess =
    div []
        [ p [] [ text "Your account is ready." ]
        , p []
            [ a [ Route.href Route.Login ] [ text "Sign in" ]
            , text " to start shopping."
            ]
        ]


viewForm : Model -> Html Msg
viewForm model =
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
            [ label [] [ text "Display name" ]
            , input
                [ type_ "text"
                , placeholder "Alice"
                , value model.name
                , onInput NameChanged
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
                [ type_ "submit"
                , disabled (not (canSubmit model))
                ]
                [ text (submitLabel model.status) ]
            ]
        , viewStatus model.status
        ]


submitLabel : Status -> String
submitLabel status =
    case status of
        Idle ->
            "Create account"

        Submitting ->
            "Creating..."

        Errored _ ->
            "Try again"

        Succeeded ->
            "Create account"


viewStatus : Status -> Html Msg
viewStatus status =
    case status of
        Idle ->
            text ""

        Submitting ->
            text ""

        Succeeded ->
            text ""

        Errored reason ->
            p [] [ text reason ]
