//! Toasty-backed persistence and transactional outbox for the Ordering bounded context.
//!
//! Public surface:
//!
//! - [`OrderRepository`]: persistence for the [`Order`](ordering_domain::Order)
//!   aggregate (insert + load + status update).
//! - [`BuyerRepository`]: persistence for the [`Buyer`](ordering_domain::Buyer)
//!   aggregate (insert + load by `identity_guid`).
//! - [`IntegrationEventLogService`]: the upstream `IntegrationEventLogEF`
//!   transactional outbox.  Callers stage events inside the same toasty
//!   [`Transaction`](toasty::Transaction) that commits the aggregate
//!   change, then a separate worker drains pending rows and publishes them
//!   to the bus.
//! - [`Error`]: hand-rolled crate error.
//!
//! `*Row` types are crate-private; the public surface speaks the
//! [`ordering_domain`] aggregates.  Toasty model registration is handled by
//! `toasty::models!(...)` at the application's `Db::builder` site, which
//! discovers our row types via the inventory submitted by `#[derive(Model)]`.

mod buyer_repository;
mod error;
mod integration_event_log;
mod mapper;
mod order_repository;
mod row;
mod time;

pub use buyer_repository::BuyerRepository;
pub use error::Error;
pub use integration_event_log::{
    EventLog, EventState, IntegrationEventLogService, PendingEventLog,
};
pub use order_repository::OrderRepository;
