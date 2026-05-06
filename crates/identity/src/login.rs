//! `POST /api/identity/login` handler.

use axum::Json;
use axum::extract::State;
use chrono::Utc;
use identity_tokens::issue;

use crate::credential::verify_password;
use crate::error::Error;
use crate::repository::UserRepository;
use crate::request::LoginRequest;
use crate::response::LoginResponse;
use crate::state::AppState;
use crate::user::User;

pub async fn handle(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, Error> {
    let (email, plaintext) = request.try_split()?;
    let user = lookup_user(&state, email.as_str()).await?;
    let password_ok = verify_password(&plaintext, user.password_hash())?;
    if password_ok {
        let now = Utc::now();
        let token = issue(
            state.issuer(),
            user.id().into_uuid(),
            user.email().as_str(),
            user.name().as_str(),
            now,
        )?;
        let ttl_seconds = state.issuer().ttl().num_seconds();
        Ok(Json(LoginResponse::bearer(token, ttl_seconds)))
    } else {
        Err(Error::InvalidCredentials)
    }
}

async fn lookup_user(state: &AppState, email: &str) -> Result<User, Error> {
    // FFI-mut exception: see `ordering-api::create_order::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let outcome = UserRepository::find_by_email(&mut tx, email).await;
    tx.commit().await?;
    // Treat "no user with that email" as InvalidCredentials so we
    // don't leak which emails are registered.
    outcome.map_err(|err| match err {
        Error::NotFound { .. } => Error::InvalidCredentials,
        other => other,
    })
}
