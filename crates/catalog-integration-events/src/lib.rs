//! Wire-form integration events for the Catalog bounded context.
//!
//! Mirror of `ordering-integration-events`, scoped to the catalog's
//! single emitted variant
//! ([`CatalogIntegrationEvent::ProductPriceChanged`]).  The wire format
//! matches upstream eShop's `Catalog.API.IntegrationEvents.Events`
//! namespace: each variant tag is the C# class name (here
//! `ProductPriceChangedIntegrationEvent`) so a downstream consumer
//! written against the upstream contract decodes the payload unchanged.
//!
//! Translation from the in-process
//! [`catalog::DomainEvent`](https://docs.rs/catalog) into a
//! [`CatalogIntegrationEvent`] lives in `catalog::outbox` rather than
//! in this crate so that the dependency arrow stays one-directional
//! (the catalog crate consumes these wire types; this crate has no
//! dependency on `catalog`).
//!
//! # Examples
//!
//! ```
//! use catalog_integration_events::{
//!     CatalogIntegrationEvent, ProductPriceChangedIntegrationEventPayload,
//! };
//! use rust_decimal::Decimal;
//!
//! let event = CatalogIntegrationEvent::ProductPriceChanged(
//!     ProductPriceChangedIntegrationEventPayload::new(
//!         42,
//!         Decimal::from(20),
//!         Decimal::from(25),
//!     ),
//! );
//! let bytes = serde_json::to_vec(&event)?;
//! let _parsed: CatalogIntegrationEvent = serde_json::from_slice(&bytes)?;
//! # Ok::<(), serde_json::Error>(())
//! ```

mod event;
mod payload;

pub use event::{CatalogIntegrationEvent, PRODUCT_PRICE_CHANGED};
pub use payload::ProductPriceChangedIntegrationEventPayload;
