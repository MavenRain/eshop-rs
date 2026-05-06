module Main exposing (main)

{-| eShop SPA entry point.

`Browser.application` owns the URL bar so `Route` updates flow
through `UrlChanged`.  Each per-page model is a variant of
[`PageModel`](#PageModel); per-page messages are wrapped through
[`PageMsg`](#PageMsg) so the dispatch in `update` stays exhaustive.

-}

import Api exposing (BaseUrl, Session)
import Browser
import Browser.Navigation as Nav
import Html exposing (Html, a, h1, main_, nav, text)
import Json.Decode as D
import Page.Basket as Basket
import Page.Catalog as Catalog
import Page.Home as Home
import Page.Login as Login
import Page.Orders as Orders
import Page.Webhooks as Webhooks
import Route exposing (Route)
import Url exposing (Url)


type alias Flags =
    { apiBase : String }


flagsDecoder : D.Decoder Flags
flagsDecoder =
    D.map Flags (D.field "apiBase" D.string)


type alias Model =
    { key : Nav.Key
    , baseUrl : BaseUrl
    , session : Session
    , page : PageModel
    }


type PageModel
    = HomeModel Home.Model
    | LoginModel Login.Model
    | CatalogModel Catalog.Model
    | BasketModel Basket.Model
    | OrdersModel Orders.Model
    | WebhooksModel Webhooks.Model


type Msg
    = UrlRequested Browser.UrlRequest
    | UrlChanged Url
    | HomeMsg Home.Msg
    | LoginMsg Login.Msg
    | CatalogMsg Catalog.Msg
    | BasketMsg Basket.Msg
    | OrdersMsg Orders.Msg
    | WebhooksMsg Webhooks.Msg


main : Program D.Value Model Msg
main =
    Browser.application
        { init = init
        , view = view
        , update = update
        , subscriptions = always Sub.none
        , onUrlRequest = UrlRequested
        , onUrlChange = UrlChanged
        }


init : D.Value -> Url -> Nav.Key -> ( Model, Cmd Msg )
init rawFlags url key =
    let
        flags =
            D.decodeValue flagsDecoder rawFlags
                |> Result.withDefault { apiBase = "http://127.0.0.1:8080" }

        baseUrl =
            Api.baseUrlFromString flags.apiBase

        session =
            Api.unauthenticated

        ( pageModel, pageCmd ) =
            initPage baseUrl session (Route.fromUrl url)
    in
    ( { key = key
      , baseUrl = baseUrl
      , session = session
      , page = pageModel
      }
    , pageCmd
    )


initPage : BaseUrl -> Session -> Route -> ( PageModel, Cmd Msg )
initPage baseUrl session route =
    case route of
        Route.Home ->
            Home.init |> mapPage HomeModel HomeMsg

        Route.Login ->
            Login.init |> mapPage LoginModel LoginMsg

        Route.Catalog ->
            Catalog.init baseUrl session |> mapPage CatalogModel CatalogMsg

        Route.Basket ->
            Basket.init baseUrl session |> mapPage BasketModel BasketMsg

        Route.Orders ->
            Orders.init baseUrl session |> mapPage OrdersModel OrdersMsg

        Route.Webhooks ->
            Webhooks.init baseUrl session |> mapPage WebhooksModel WebhooksMsg


mapPage : (subModel -> PageModel) -> (subMsg -> Msg) -> ( subModel, Cmd subMsg ) -> ( PageModel, Cmd Msg )
mapPage toPageModel toPageMsg ( subModel, subCmd ) =
    ( toPageModel subModel, Cmd.map toPageMsg subCmd )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model.page ) of
        ( UrlRequested (Browser.Internal url), _ ) ->
            ( model, Nav.pushUrl model.key (Url.toString url) )

        ( UrlRequested (Browser.External href), _ ) ->
            ( model, Nav.load href )

        ( UrlChanged url, _ ) ->
            let
                ( pageModel, pageCmd ) =
                    initPage model.baseUrl model.session (Route.fromUrl url)
            in
            ( { model | page = pageModel }, pageCmd )

        ( HomeMsg subMsg, HomeModel subModel ) ->
            Home.update subMsg subModel
                |> mapPage HomeModel HomeMsg
                |> attachToModel model

        ( LoginMsg subMsg, LoginModel subModel ) ->
            let
                ( updatedSubModel, subCmd ) =
                    Login.update model.baseUrl model.session subMsg subModel

                ( consumedSubModel, maybeToken ) =
                    Login.consumeToken updatedSubModel

                nextSession =
                    maybeToken
                        |> Maybe.map Api.withToken
                        |> Maybe.withDefault model.session
            in
            ( { model
                | page = LoginModel consumedSubModel
                , session = nextSession
              }
            , Cmd.map LoginMsg subCmd
            )

        ( CatalogMsg subMsg, CatalogModel subModel ) ->
            Catalog.update subMsg subModel
                |> mapPage CatalogModel CatalogMsg
                |> attachToModel model

        ( BasketMsg subMsg, BasketModel subModel ) ->
            Basket.update subMsg subModel
                |> mapPage BasketModel BasketMsg
                |> attachToModel model

        ( OrdersMsg subMsg, OrdersModel subModel ) ->
            Orders.update subMsg subModel
                |> mapPage OrdersModel OrdersMsg
                |> attachToModel model

        ( WebhooksMsg subMsg, WebhooksModel subModel ) ->
            Webhooks.update subMsg subModel
                |> mapPage WebhooksModel WebhooksMsg
                |> attachToModel model

        -- Page message arrived for a different page (URL changed
        -- before the message was processed).  Drop it.
        ( HomeMsg _, _ ) ->
            ( model, Cmd.none )

        ( LoginMsg _, _ ) ->
            ( model, Cmd.none )

        ( CatalogMsg _, _ ) ->
            ( model, Cmd.none )

        ( BasketMsg _, _ ) ->
            ( model, Cmd.none )

        ( OrdersMsg _, _ ) ->
            ( model, Cmd.none )

        ( WebhooksMsg _, _ ) ->
            ( model, Cmd.none )


attachToModel : Model -> ( PageModel, Cmd Msg ) -> ( Model, Cmd Msg )
attachToModel model ( pageModel, pageCmd ) =
    ( { model | page = pageModel }, pageCmd )


view : Model -> Browser.Document Msg
view model =
    { title = "eShop"
    , body =
        [ viewNav
        , main_ []
            [ h1 [] [ text "eShop" ]
            , viewPage model.page
            ]
        ]
    }


viewNav : Html Msg
viewNav =
    nav []
        [ a [ Route.href Route.Home ] [ text "Home" ]
        , a [ Route.href Route.Catalog ] [ text "Catalog" ]
        , a [ Route.href Route.Basket ] [ text "Basket" ]
        , a [ Route.href Route.Orders ] [ text "Orders" ]
        , a [ Route.href Route.Webhooks ] [ text "Webhooks" ]
        , a [ Route.href Route.Login ] [ text "Sign in" ]
        ]


viewPage : PageModel -> Html Msg
viewPage page =
    case page of
        HomeModel subModel ->
            Home.view subModel |> Html.map HomeMsg

        LoginModel subModel ->
            Login.view subModel |> Html.map LoginMsg

        CatalogModel subModel ->
            Catalog.view subModel |> Html.map CatalogMsg

        BasketModel subModel ->
            Basket.view subModel |> Html.map BasketMsg

        OrdersModel subModel ->
            Orders.view subModel |> Html.map OrdersMsg

        WebhooksModel subModel ->
            Webhooks.view subModel |> Html.map WebhooksMsg
