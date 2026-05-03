//! Convert in-memory [`DomainEvent`]s into outbox rows.

use uuid::Uuid;

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
/// The `content` column carries a debug rendering of the event for
/// v1; a production deployment would route through `serde_json` once
/// the [`DomainEvent`] enum derives `Serialize`/`Deserialize`.  Same
/// caveat as `ordering-api::outbox`.
#[must_use]
pub fn domain_event_to_pending(event: &DomainEvent, transaction_id: Uuid) -> PendingEventLog {
    let event_id = Uuid::new_v4();
    let creation_time = chrono::Utc::now();
    let event_type_name = event_type_name(event).to_string();
    let content = format!("{event:?}");
    PendingEventLog::new(
        event_id,
        event_type_name,
        creation_time,
        content,
        transaction_id,
    )
}

fn event_type_name(event: &DomainEvent) -> &'static str {
    match event {
        DomainEvent::ProductPriceChanged(_) => "ProductPriceChanged",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    use crate::error::Error;
    use crate::event::ProductPriceChangedEvent;
    use crate::item::CatalogItemId;
    use crate::money::Price;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    #[test]
    fn pending_carries_supplied_transaction_id() -> Result<(), Error> {
        let txn = Uuid::new_v4();
        let event = DomainEvent::ProductPriceChanged(ProductPriceChangedEvent::new(
            CatalogItemId::new(),
            Price::new(Decimal::from(25))?,
            Price::new(Decimal::from(20))?,
        ));
        let pending = domain_event_to_pending(&event, txn);
        check(pending.transaction_id() == txn, || {
            "transaction_id mismatch".to_string()
        })?;
        check(pending.event_type_name() == "ProductPriceChanged", || {
            format!("name {}", pending.event_type_name())
        })
    }

    #[test]
    fn event_type_name_dispatches() -> Result<(), Error> {
        let event = DomainEvent::ProductPriceChanged(ProductPriceChangedEvent::new(
            CatalogItemId::new(),
            Price::new(Decimal::from(1))?,
            Price::new(Decimal::from(2))?,
        ));
        check(event_type_name(&event) == "ProductPriceChanged", || {
            format!("got {}", event_type_name(&event))
        })
    }
}
