//! Wire-form integration events for the Basket bounded context.
//!
//! Mirror of `ordering-integration-events` and
//! `catalog-integration-events`, scoped to the basket's single emitted
//! variant
//! ([`BasketIntegrationEvent::UserCheckoutAccepted`]).  The wire
//! format matches upstream eShop's
//! `Basket.API.IntegrationEvents.Events` namespace; the
//! variant tag is the C# class name
//! (`UserCheckoutAcceptedIntegrationEvent`) so a downstream consumer
//! written against the upstream contract decodes the payload
//! unchanged.
//!
//! Translation from the in-process `basket::DomainEvent` lives in
//! `basket::outbox` rather than in this crate so the dependency arrow
//! stays one-directional (the basket crate consumes these wire types;
//! this crate has no dependency on `basket`).
//!
//! # Examples
//!
//! ```
//! use basket_integration_events::{
//!     BasketIntegrationEvent, UserCheckoutAcceptedIntegrationEventPayload,
//! };
//! use uuid::Uuid;
//!
//! let event = BasketIntegrationEvent::UserCheckoutAccepted(
//!     UserCheckoutAcceptedIntegrationEventPayload::new(
//!         Uuid::nil(),
//!         Uuid::nil(),
//!         Vec::new(),
//!     ),
//! );
//! let bytes = serde_json::to_vec(&event)?;
//! let _parsed: BasketIntegrationEvent = serde_json::from_slice(&bytes)?;
//! # Ok::<(), serde_json::Error>(())
//! ```

mod event;
mod payload;

pub use event::{BasketIntegrationEvent, USER_CHECKOUT_ACCEPTED};
pub use payload::{BasketSnapshotItem, UserCheckoutAcceptedIntegrationEventPayload};
