//! `POST /api/basket/checkout` handler.
//!
//! Loads the basket for the authenticated principal, validates it
//! is non-empty, deletes it, writes a
//! [`UserCheckoutAcceptedIntegrationEvent`](basket_integration_events::USER_CHECKOUT_ACCEPTED)
//! row to the outbox, and returns the basket snapshot.  The basket
//! delete and the outbox write commit in the same toasty transaction;
//! the `basket-processor` worker drains the row and forwards it onto
//! the bus.

use axum::Json;
use axum::extract::State;
use identity_middleware::AuthenticatedPrincipal;
use uuid::Uuid;

use crate::customer::CustomerId;
use crate::error::Error;
use crate::event::{CheckoutAcceptedEvent, DomainEvent};
use crate::integration_event_log::{IntegrationEventLogService, PendingEventLog};
use crate::outbox::domain_event_to_pending;
use crate::repository::BasketRepository;
use crate::response::BasketResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    principal: AuthenticatedPrincipal,
) -> Result<Json<BasketResponse>, Error> {
    let id = CustomerId::from(principal.principal().user_id());
    // FFI-mut exception: see `update_basket::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let basket = BasketRepository::get_by_customer_id(&mut tx, id).await?;
    if basket.is_empty() {
        Err(Error::Validation {
            reason: "cannot checkout an empty basket".to_string(),
        })
    } else {
        let request_id = Uuid::new_v4();
        let domain_event = DomainEvent::CheckoutAccepted(CheckoutAcceptedEvent::new(
            id,
            request_id,
            basket.items().to_vec(),
        ));
        BasketRepository::delete_by_customer_id(&mut tx, id).await?;
        // `filter_map` + `Result::transpose` drops events without an
        // integration counterpart and propagates serialization errors;
        // `save_events_batch` then performs the sequential insert
        // behind a single API call (parity with the ordering and
        // catalog handlers).
        let transaction_id = Uuid::new_v4();
        let pendings: Vec<PendingEventLog> = [domain_event]
            .iter()
            .filter_map(|event| domain_event_to_pending(event, transaction_id).transpose())
            .collect::<Result<Vec<_>, _>>()?;
        IntegrationEventLogService::save_events_batch(&mut tx, pendings).await?;
        tx.commit().await?;
        Ok(Json(BasketResponse::from(&basket)))
    }
}
