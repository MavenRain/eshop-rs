//! JWT issuance and validation primitives for the Identity bounded
//! context.
//!
//! The Identity service uses [`issuer::issue`] to mint a JWT after
//! verifying a user's credentials.  Other services depend on this
//! crate (without the heavier `identity` aggregate / persistence
//! deps) and call [`validator::validate`] in their auth middleware
//! to project a bearer token into a [`Principal`].
//!
//! Symmetric (HMAC) signing for v1: producer and consumers share
//! the same secret, supplied via [`IssuerConfig`] / [`ValidatorConfig`].
//! A production deployment would migrate to asymmetric signing
//! (RSA / ECDSA) so consumers can verify without holding the issuer
//! key; that swap is purely a config-level change with this crate.

pub mod claims;
pub mod error;
pub mod issuer;
pub mod validator;

pub use claims::{Claims, Principal, claims_to_principal};
pub use error::Error;
pub use issuer::{IssuerConfig, issue};
pub use validator::{ValidatorConfig, validate};
