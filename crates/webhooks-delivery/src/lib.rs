//! Delivery worker for the Webhooks bounded context.
//!
//! Subscribes to integration events emitted by the other bounded
//! contexts (ordering / catalog / basket) and POSTs each delivered
//! event to every webhook subscription whose
//! [`webhook_type`](webhooks::WebhookSubscription::webhook_type)
//! matches the event's wire name.
//!
//! [`make_handler`] is generic over the integration-event type, so a
//! single implementation services all three context buses.  The
//! `AppHost` instantiates one closure per bus and drives them via
//! [`event_bus::EventBusSubscriber::subscribe`].
//!
//! Per-delivery HTTP failures and non-2xx responses are logged but
//! never fail the batch.  Retry / dead-letter machinery (with
//! exponential backoff and a delivery-attempt log) is a future
//! slice; v1 is best-effort.

pub mod error;
mod handler;

pub use error::Error;
pub use handler::make_handler;
