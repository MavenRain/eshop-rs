//! Generic delivery handler factory.
//!
//! [`make_handler`] returns a closure that, when an integration
//! event of type `E` arrives:
//!
//! 1. Serializes the event to JSON.
//! 2. Looks up subscriptions whose `webhook_type` matches
//!    [`E::event_name`](event_bus::IntegrationEvent::event_name).
//! 3. Issues a `POST` to each subscription's `destination_url` with
//!    the JSON body and an `Authorization: Bearer <token>` header.
//!
//! Persistence errors propagate (and surface as
//! [`event_bus::Error::Subscribe`] at the bus boundary).  Per-delivery
//! HTTP errors and non-2xx responses are logged to stderr but do not
//! fail the batch — webhook delivery is best-effort in v1; retry /
//! dead-letter machinery is a future slice.
//!
//! ## Async-driving inside an `Io::suspend` body
//!
//! Same approach as `catalog-subscribers` and `ordering-subscribers`:
//! the bus drives the handler synchronously via
//! `handler(event).run()`, so the body uses
//! [`tokio::task::block_in_place`] +
//! [`tokio::runtime::Handle::current`]`.block_on(...)` to drive both
//! the toasty lookup and the reqwest POST on the bus's owned
//! multi-threaded runtime.

use std::sync::Arc;

use comp_cat_rs::effect::io::Io;
use event_bus::{Error as BusError, IntegrationEvent};
use toasty::Db;
use webhooks::{WebhookSubscription, WebhookSubscriptionRepository};

use crate::error::Error;

/// Build a delivery handler closure for events of type `E`.
///
/// The returned closure captures the shared toasty [`Db`] handle and
/// the shared [`reqwest::Client`].  Drive it with
/// `bus.subscribe(handler).run()?` at `AppHost` startup.
pub fn make_handler<E>(
    db: Arc<Db>,
    http_client: Arc<reqwest::Client>,
) -> impl Fn(E) -> Io<BusError, ()> + Send + Sync + 'static
where
    E: IntegrationEvent,
{
    move |event| {
        let db = db.clone();
        let http_client = http_client.clone();
        Io::suspend(move || run_blocking(db, http_client, event).map_err(BusError::from))
    }
}

fn run_blocking<E>(db: Arc<Db>, http_client: Arc<reqwest::Client>, event: E) -> Result<(), Error>
where
    E: IntegrationEvent,
{
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async move { dispatch(db, http_client, event).await })
    })
}

async fn dispatch<E>(db: Arc<Db>, http_client: Arc<reqwest::Client>, event: E) -> Result<(), Error>
where
    E: IntegrationEvent,
{
    let event_name = event.event_name().to_string();
    let payload = serde_json::to_string(&event)?;
    let subscriptions = fetch_subscriptions(&db, &event_name).await?;
    deliver_to_subscriptions(&http_client, &subscriptions, &payload).await;
    Ok(())
}

async fn fetch_subscriptions(db: &Db, event_name: &str) -> Result<Vec<WebhookSubscription>, Error> {
    // FFI-mut exception: toasty's Connection::transaction takes
    // `&mut self`; same exception class as the API-layer handlers.
    let mut conn = db.connection().await?;
    let mut tx = conn.transaction().await?;
    let subscriptions = WebhookSubscriptionRepository::list_by_type(&mut tx, event_name).await?;
    tx.commit().await?;
    Ok(subscriptions)
}

/// Sequential-async delivery, parallel to the catalog
/// `decrement_stock_for_items` helper: the FFI-async `for` loop is
/// localized here so the call site stays combinator-shaped.  Each
/// subscription's HTTP outcome is logged but never fails the batch.
async fn deliver_to_subscriptions(
    http_client: &reqwest::Client,
    subscriptions: &[WebhookSubscription],
    payload: &str,
) {
    for subscription in subscriptions {
        deliver_one(http_client, subscription, payload).await;
    }
}

async fn deliver_one(
    http_client: &reqwest::Client,
    subscription: &WebhookSubscription,
    payload: &str,
) {
    let url = subscription.destination_url().as_str();
    let token = subscription.token().as_str();
    let outcome = http_client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .body(payload.to_string())
        .send()
        .await;
    match outcome {
        Ok(response) => log_response(url, &response),
        Err(err) => eprintln!("[webhooks-delivery] {url} delivery error: {err}"),
    }
}

fn log_response(url: &str, response: &reqwest::Response) {
    let status = response.status();
    if !status.is_success() {
        eprintln!("[webhooks-delivery] {url} returned {status}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Json { reason: reason() })
        }
    }

    #[test]
    fn error_maps_to_subscribe_at_bus_boundary() -> Result<(), Error> {
        let err: BusError = Error::Toasty {
            reason: "x".to_string(),
        }
        .into();
        check(matches!(err, BusError::Subscribe { .. }), || {
            format!("got {err:?}")
        })
    }

    #[test]
    fn json_error_propagates_through_from() -> Result<(), Error> {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str("not json");
        let outcome = parsed.map_err(Error::from);
        check(matches!(outcome, Err(Error::Json { .. })), || {
            format!("got {outcome:?}")
        })
    }
}
