module Api exposing
    ( BaseUrl
    , Session
    , baseUrlFromString
    , baseUrlToString
    , delete
    , get
    , post
    , put
    , unauthenticated
    , withToken
    )

{-| Talk-to-apphost layer.

`BaseUrl` is a thin newtype around the apphost root (e.g.
`http://127.0.0.1:8080`).  `Session` carries an optional bearer
token; `get` / `post` thread it onto the `Authorization` header
when present.

The page modules call `Api.get` / `Api.post`; they don't construct
`Http.Request` values directly so the auth header is enforced in one
place.

-}

import Http
import Json.Decode as D
import Json.Encode as E


{-| Base URL of the eshop apphost, opaque newtype.
-}
type BaseUrl
    = BaseUrl String


{-| Build from a plain string.  Trailing slashes are stripped so the
path-join below stays simple.
-}
baseUrlFromString : String -> BaseUrl
baseUrlFromString raw =
    let
        trimmed =
            if String.endsWith "/" raw then
                String.dropRight 1 raw

            else
                raw
    in
    BaseUrl trimmed


baseUrlToString : BaseUrl -> String
baseUrlToString (BaseUrl s) =
    s


{-| Per-tab authentication state.  `Nothing` means anonymous; some
endpoints will then return 401, which the page modules handle.
-}
type Session
    = Session (Maybe String)


unauthenticated : Session
unauthenticated =
    Session Nothing


withToken : String -> Session
withToken token =
    Session (Just token)


authHeaders : Session -> List Http.Header
authHeaders (Session token) =
    case token of
        Just t ->
            [ Http.header "Authorization" ("Bearer " ++ t) ]

        Nothing ->
            []


{-| GET `<base>/<path>` decoding the JSON body via `decoder`.
-}
get :
    BaseUrl
    -> Session
    -> String
    -> D.Decoder a
    -> (Result Http.Error a -> msg)
    -> Cmd msg
get base session path decoder toMsg =
    Http.request
        { method = "GET"
        , headers = authHeaders session
        , url = baseUrlToString base ++ path
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg decoder
        , timeout = Nothing
        , tracker = Nothing
        }


{-| POST `<base>/<path>` with a JSON body, decoding the JSON
response.
-}
post :
    BaseUrl
    -> Session
    -> String
    -> E.Value
    -> D.Decoder a
    -> (Result Http.Error a -> msg)
    -> Cmd msg
post base session path body decoder toMsg =
    Http.request
        { method = "POST"
        , headers = authHeaders session
        , url = baseUrlToString base ++ path
        , body = Http.jsonBody body
        , expect = Http.expectJson toMsg decoder
        , timeout = Nothing
        , tracker = Nothing
        }


{-| PUT `<base>/<path>` with a JSON body, decoding the JSON
response.
-}
put :
    BaseUrl
    -> Session
    -> String
    -> E.Value
    -> D.Decoder a
    -> (Result Http.Error a -> msg)
    -> Cmd msg
put base session path body decoder toMsg =
    Http.request
        { method = "PUT"
        , headers = authHeaders session
        , url = baseUrlToString base ++ path
        , body = Http.jsonBody body
        , expect = Http.expectJson toMsg decoder
        , timeout = Nothing
        , tracker = Nothing
        }


{-| DELETE `<base>/<path>`.  Used for endpoints returning 204 No
Content; the result type carries no body.
-}
delete :
    BaseUrl
    -> Session
    -> String
    -> (Result Http.Error () -> msg)
    -> Cmd msg
delete base session path toMsg =
    Http.request
        { method = "DELETE"
        , headers = authHeaders session
        , url = baseUrlToString base ++ path
        , body = Http.emptyBody
        , expect = Http.expectWhatever toMsg
        , timeout = Nothing
        , tracker = Nothing
        }
