//! Convert in-memory [`DomainEvent`]s into outbox rows.
//!
//! Each domain event is first translated to a wire-form
//! [`CatalogIntegrationEvent`] and then JSON-serialized into the
//! `content` column.  The `catalog-processor` worker reads the column
//! back, deserializes it, and forwards the integration event onto the
//! bus.
//!
//! Translation lives here rather than in `catalog-integration-events`
//! so the dependency arrow stays one-directional: this crate (catalog)
//! depends on `catalog-integration-events`, not vice versa.

use catalog_integration_events::{
    CatalogIntegrationEvent, ProductPriceChangedIntegrationEventPayload,
};
use event_bus::IntegrationEvent;
use uuid::Uuid;

use crate::error::Error;
use crate::event::DomainEvent;
use crate::integration_event_log::PendingEventLog;

/// Project a single [`DomainEvent`] into a [`PendingEventLog`] row,
/// ready to be inserted alongside the aggregate change in the same
/// transaction.
///
/// `transaction_id` is the application-supplied correlator that ties
/// every outbox row written in a single transaction together; the
/// publisher worker pulls all rows for a given `transaction_id` once
/// the originating transaction commits.
///
/// Returns [`Ok(None)`] when the domain event has no integration
/// counterpart; the caller skips the outbox insert in that case.
/// Every catalog domain variant currently maps to [`Some`], but the
/// signature is reserved for forward compatibility (mirrors
/// `ordering-api::outbox::domain_event_to_pending`, where
/// `BuyerPaymentMethodVerified` returns [`None`]).
///
/// # Errors
/// Returns [`Error::Json`] if the integration event fails to serialize.
pub fn domain_event_to_pending(
    event: &DomainEvent,
    transaction_id: Uuid,
) -> Result<Option<PendingEventLog>, Error> {
    from_domain_event(event)
        .map(|integration| {
            let content = serde_json::to_string(&integration)?;
            Ok(PendingEventLog::new(
                Uuid::new_v4(),
                integration.event_name().to_string(),
                chrono::Utc::now(),
                content,
                transaction_id,
            ))
        })
        .transpose()
}

/// Project a domain event into its wire-form integration counterpart,
/// if one exists.
///
/// Currently every catalog domain variant maps to [`Some`]; the
/// [`Option`] wrap is reserved for forward compatibility with future
/// variants that may stay in-process (parity with
/// `ordering-api::outbox::from_domain_event`, where
/// `BuyerPaymentMethodVerified` returns [`None`]).
#[allow(clippy::unnecessary_wraps)]
#[must_use]
pub fn from_domain_event(event: &DomainEvent) -> Option<CatalogIntegrationEvent> {
    match event {
        DomainEvent::ProductPriceChanged(payload) => {
            Some(CatalogIntegrationEvent::ProductPriceChanged(
                ProductPriceChangedIntegrationEventPayload::new(
                    payload.product_id().into_uuid(),
                    payload.new_price().into_decimal(),
                    payload.old_price().into_decimal(),
                ),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use catalog_integration_events::PRODUCT_PRICE_CHANGED;
    use rust_decimal::Decimal;

    use crate::event::ProductPriceChangedEvent;
    use crate::item::CatalogItemId;
    use crate::money::Price;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        match () {
            () if cond => Ok(()),
            () => Err(Error::Validation { reason: reason() }),
        }
    }

    fn sample_domain_event() -> Result<DomainEvent, Error> {
        Ok(DomainEvent::ProductPriceChanged(
            ProductPriceChangedEvent::new(
                CatalogItemId::new(),
                Price::new(Decimal::from(20))?,
                Price::new(Decimal::from(25))?,
            ),
        ))
    }

    #[test]
    fn pending_carries_supplied_transaction_id() -> Result<(), Error> {
        let txn = Uuid::new_v4();
        let pending = domain_event_to_pending(&sample_domain_event()?, txn)?.ok_or_else(|| {
            Error::Validation {
                reason: "ProductPriceChanged should produce a pending row".to_string(),
            }
        })?;
        check(pending.transaction_id() == txn, || {
            "transaction_id mismatch".to_string()
        })?;
        check(pending.event_type_name() == PRODUCT_PRICE_CHANGED, || {
            format!("name {}", pending.event_type_name())
        })
    }

    #[test]
    fn pending_content_round_trips_to_integration_event() -> Result<(), Error> {
        let txn = Uuid::new_v4();
        let domain = sample_domain_event()?;
        let pending = domain_event_to_pending(&domain, txn)?.ok_or_else(|| Error::Validation {
            reason: "ProductPriceChanged should produce a pending row".to_string(),
        })?;
        let parsed: CatalogIntegrationEvent = serde_json::from_str(pending.content())?;
        let CatalogIntegrationEvent::ProductPriceChanged(payload) = &parsed;
        check(payload.new_price() == Decimal::from(20), || {
            format!("new_price {}", payload.new_price())
        })?;
        check(payload.old_price() == Decimal::from(25), || {
            format!("old_price {}", payload.old_price())
        })
    }

    #[test]
    fn from_domain_event_translates_product_price_changed() -> Result<(), Error> {
        let domain = sample_domain_event()?;
        let outcome = from_domain_event(&domain);
        check(outcome.is_some(), || format!("got {outcome:?}"))
    }
}
