module Page.Webhooks exposing (Model, Msg, init, update, view)

{-| Webhooks management screen.

The page lists subscriptions owned by the authenticated caller
(`GET /api/webhooks`), supports registering a new subscription
(`POST /api/webhooks`), and supports deleting one
(`DELETE /api/webhooks/{id}`).  Every endpoint requires a JWT
bearer token, attached automatically through [`Api`](Api).

State machine:

  - `Loading` — initial fetch in flight.
  - `Loaded subs` — list rendered, register form active.
  - `Failed reason` — fetch failed; surfaces a message.

The `Draft` carried alongside the list state holds the in-progress
new-subscription form so the user can keep typing while a fetch
or a delete is in flight.

-}

import Api exposing (BaseUrl, Session)
import Html exposing (Html, button, div, em, form, h2, input, label, li, p, strong, text, ul)
import Html.Attributes as Attr
import Html.Events as Events
import Http
import Json.Decode as D
import Json.Encode as E


type alias Model =
    { state : ListState
    , draft : Draft
    , baseUrl : BaseUrl
    , session : Session
    }


type ListState
    = Loading
    | Loaded (List Subscription)
    | Failed String


type alias Subscription =
    { id : String
    , webhookType : String
    , destinationUrl : String
    , createdAt : String
    }


type alias Draft =
    { webhookType : String
    , destinationUrl : String
    , token : String
    , status : DraftStatus
    }


type DraftStatus
    = DraftIdle
    | DraftSubmitting
    | DraftErrored String


type Msg
    = GotSubscriptions (Result Http.Error (List Subscription))
    | WebhookTypeChanged String
    | DestinationUrlChanged String
    | TokenChanged String
    | RegisterSubmitted
    | Registered (Result Http.Error String)
    | DeleteClicked String
    | Deleted (Result Http.Error ())


init : BaseUrl -> Session -> ( Model, Cmd Msg )
init baseUrl session =
    ( { state = Loading
      , draft = emptyDraft
      , baseUrl = baseUrl
      , session = session
      }
    , fetchSubscriptions baseUrl session
    )


emptyDraft : Draft
emptyDraft =
    { webhookType = ""
    , destinationUrl = ""
    , token = ""
    , status = DraftIdle
    }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotSubscriptions (Ok subs) ->
            ( { model | state = Loaded subs }, Cmd.none )

        GotSubscriptions (Err err) ->
            ( { model | state = Failed (errorToString err) }, Cmd.none )

        WebhookTypeChanged value ->
            ( { model | draft = updateDraftType value model.draft }, Cmd.none )

        DestinationUrlChanged value ->
            ( { model | draft = updateDraftUrl value model.draft }, Cmd.none )

        TokenChanged value ->
            ( { model | draft = updateDraftToken value model.draft }, Cmd.none )

        RegisterSubmitted ->
            if canSubmit model.draft then
                ( { model | draft = setDraftStatus DraftSubmitting model.draft }
                , registerSubscription model
                )

            else
                ( model, Cmd.none )

        Registered (Ok _) ->
            ( { model | draft = emptyDraft }
            , fetchSubscriptions model.baseUrl model.session
            )

        Registered (Err err) ->
            ( { model | draft = setDraftStatus (DraftErrored (errorToString err)) model.draft }
            , Cmd.none
            )

        DeleteClicked id ->
            ( model, deleteSubscription model id )

        Deleted (Ok ()) ->
            ( model, fetchSubscriptions model.baseUrl model.session )

        Deleted (Err err) ->
            ( { model | state = Failed (errorToString err) }, Cmd.none )


updateDraftType : String -> Draft -> Draft
updateDraftType value draft =
    { draft | webhookType = value, status = DraftIdle }


updateDraftUrl : String -> Draft -> Draft
updateDraftUrl value draft =
    { draft | destinationUrl = value, status = DraftIdle }


updateDraftToken : String -> Draft -> Draft
updateDraftToken value draft =
    { draft | token = value, status = DraftIdle }


setDraftStatus : DraftStatus -> Draft -> Draft
setDraftStatus status draft =
    { draft | status = status }


canSubmit : Draft -> Bool
canSubmit draft =
    let
        nonEmpty =
            String.trim >> String.isEmpty >> not
    in
    nonEmpty draft.webhookType
        && nonEmpty draft.destinationUrl
        && nonEmpty draft.token
        && draft.status
        /= DraftSubmitting


fetchSubscriptions : BaseUrl -> Session -> Cmd Msg
fetchSubscriptions baseUrl session =
    Api.get baseUrl session "/api/webhooks" subscriptionsDecoder GotSubscriptions


registerSubscription : Model -> Cmd Msg
registerSubscription model =
    Api.post
        model.baseUrl
        model.session
        "/api/webhooks"
        (encodeRegister model.draft)
        registeredDecoder
        Registered


deleteSubscription : Model -> String -> Cmd Msg
deleteSubscription model id =
    Api.delete model.baseUrl model.session ("/api/webhooks/" ++ id) Deleted


encodeRegister : Draft -> E.Value
encodeRegister draft =
    E.object
        [ ( "webhook_type", E.string draft.webhookType )
        , ( "destination_url", E.string draft.destinationUrl )
        , ( "token", E.string draft.token )
        ]


subscriptionsDecoder : D.Decoder (List Subscription)
subscriptionsDecoder =
    D.list subscriptionDecoder


subscriptionDecoder : D.Decoder Subscription
subscriptionDecoder =
    D.map4 Subscription
        (D.field "id" D.string)
        (D.field "webhook_type" D.string)
        (D.field "destination_url" D.string)
        (D.field "created_at" D.string)


registeredDecoder : D.Decoder String
registeredDecoder =
    D.field "id" D.string


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
        [ h2 [] [ text "Webhooks" ]
        , viewRegisterForm model.draft
        , viewListState model.state
        ]


viewRegisterForm : Draft -> Html Msg
viewRegisterForm draft =
    form [ Events.onSubmit RegisterSubmitted ]
        [ p [] [ strong [] [ text "Register a new subscription" ] ]
        , div []
            [ label []
                [ text "Event type"
                , input
                    [ Attr.type_ "text"
                    , Attr.value draft.webhookType
                    , Attr.placeholder "OrderShippedIntegrationEvent"
                    , Events.onInput WebhookTypeChanged
                    ]
                    []
                ]
            ]
        , div []
            [ label []
                [ text "Destination URL"
                , input
                    [ Attr.type_ "url"
                    , Attr.value draft.destinationUrl
                    , Attr.placeholder "https://your.site/hook"
                    , Events.onInput DestinationUrlChanged
                    ]
                    []
                ]
            ]
        , div []
            [ label []
                [ text "Token"
                , input
                    [ Attr.type_ "text"
                    , Attr.value draft.token
                    , Attr.placeholder "shared secret"
                    , Events.onInput TokenChanged
                    ]
                    []
                ]
            ]
        , button
            [ Attr.type_ "submit"
            , Attr.disabled (not (canSubmit draft))
            ]
            [ text (registerButtonLabel draft.status) ]
        , viewDraftStatus draft.status
        ]


registerButtonLabel : DraftStatus -> String
registerButtonLabel status =
    case status of
        DraftSubmitting ->
            "Registering..."

        DraftIdle ->
            "Register"

        DraftErrored _ ->
            "Register"


viewDraftStatus : DraftStatus -> Html msg
viewDraftStatus status =
    case status of
        DraftIdle ->
            text ""

        DraftSubmitting ->
            text ""

        DraftErrored reason ->
            p [] [ em [] [ text ("Could not register: " ++ reason) ] ]


viewListState : ListState -> Html Msg
viewListState state =
    case state of
        Loading ->
            p [] [ text "Loading subscriptions..." ]

        Failed reason ->
            p [] [ em [] [ text ("Could not load subscriptions: " ++ reason) ] ]

        Loaded [] ->
            p [] [ em [] [ text "No subscriptions registered yet." ] ]

        Loaded subs ->
            div []
                [ p [] [ strong [] [ text "Active subscriptions" ] ]
                , ul [] (List.map viewSubscription subs)
                ]


viewSubscription : Subscription -> Html Msg
viewSubscription sub =
    li []
        [ strong [] [ text sub.webhookType ]
        , text " -> "
        , text sub.destinationUrl
        , text " (registered "
        , text sub.createdAt
        , text ") "
        , button
            [ Events.onClick (DeleteClicked sub.id) ]
            [ text "Delete" ]
        ]
