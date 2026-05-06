module Route exposing (Route(..), fromUrl, href, toString)

{-| Application routes.  Adding a new screen is two edits: a new
[`Route`](#Route) variant and a matching [`UrlParser.Parser`](#parser)
arm.
-}

import Html exposing (Attribute)
import Html.Attributes
import Url exposing (Url)
import Url.Parser as UrlParser exposing ((</>), Parser)


type Route
    = Home
    | Login
    | Catalog
    | Basket


parser : Parser (Route -> a) a
parser =
    UrlParser.oneOf
        [ UrlParser.map Home UrlParser.top
        , UrlParser.map Login (UrlParser.s "login")
        , UrlParser.map Catalog (UrlParser.s "catalog")
        , UrlParser.map Basket (UrlParser.s "basket")
        ]


fromUrl : Url -> Route
fromUrl url =
    UrlParser.parse parser url
        |> Maybe.withDefault Home


toString : Route -> String
toString route =
    case route of
        Home ->
            "/"

        Login ->
            "/login"

        Catalog ->
            "/catalog"

        Basket ->
            "/basket"


{-| Convenience for building `<a href>` attributes; just `href` to
keep call sites short.
-}
href : Route -> Attribute msg
href route =
    Html.Attributes.href (toString route)
