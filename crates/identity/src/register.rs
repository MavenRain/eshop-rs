//! `POST /api/identity/register` handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use chrono::Utc;

use crate::credential::hash_password;
use crate::error::Error;
use crate::repository::UserRepository;
use crate::request::RegisterRequest;
use crate::response::RegisteredUserResponse;
use crate::state::AppState;
use crate::user::{User, UserId};

pub async fn handle(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisteredUserResponse>), Error> {
    let (email, name, plaintext) = request.try_split()?;
    let password_hash = hash_password(&plaintext)?;
    let user = User::new(UserId::new(), email, name, password_hash, Utc::now());
    let user_id = user.id();
    // FFI-mut exception: see `ordering-api::create_order::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    UserRepository::add(&mut tx, &user).await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(RegisteredUserResponse::new(user_id.into_uuid())),
    ))
}
