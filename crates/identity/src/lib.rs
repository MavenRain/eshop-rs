//! Identity bounded context: user accounts, password credentials,
//! and JWT issuance via [`identity_tokens`].
//!
//! V1 surface:
//!
//! - `POST /api/identity/register`: create a new user with an
//!   email + plaintext password.  Password is hashed with argon2;
//!   the response is `201 Created` + `{ id }`.
//! - `POST /api/identity/login`: verify email + password, return a
//!   signed JWT.
//!
//! The validation crate ([`identity_tokens`]) is the lightweight dep
//! every other service will pull in for auth middleware (slice 4B);
//! this `identity` crate stays out of those services' dep trees.

mod credential;
pub mod error;
mod login;
pub mod mapper;
mod register;
mod repository;
pub mod request;
pub mod response;
pub mod row;
mod state;
pub mod strings;
pub mod time;
pub mod user;

pub use error::Error;
pub use repository::UserRepository;
pub use request::{LoginRequest, RegisterRequest};
pub use response::{LoginResponse, RegisteredUserResponse};
pub use state::AppState;
pub use strings::{Email, PasswordHash, UserName};
pub use user::{User, UserId};

use axum::Router;
use axum::routing::post;

/// Build the axum [`Router`] for the Identity API, with `state` injected.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/identity/register", post(register::handle))
        .route("/api/identity/login", post(login::handle))
        .with_state(state)
}
