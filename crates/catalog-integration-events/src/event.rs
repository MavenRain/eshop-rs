//! [`CatalogIntegrationEvent`] sum type and its
//! [`IntegrationEvent`] implementation.
//!
//! The wire format mirrors upstream eShop's `Catalog.API.IntegrationEvents`
//! namespace: an internally-tagged enum with the variant tag renamed to
//! the upstream C# class name (`ProductPriceChangedIntegrationEvent`).

use event_bus::IntegrationEvent;
use serde::{Deserialize, Serialize};

use crate::payload::ProductPriceChangedIntegrationEventPayload;

/// Stable wire name for [`CatalogIntegrationEvent::ProductPriceChanged`].
pub const PRODUCT_PRICE_CHANGED: &str = "ProductPriceChangedIntegrationEvent";

const ALL_EVENT_NAMES: &[&str] = &[PRODUCT_PRICE_CHANGED];

/// Integration events emitted by the Catalog bounded context.
///
/// Single variant for v1, mirroring upstream's single emission from
/// `Catalog.API`.  Forward-compatible: more variants will appear as
/// the catalog gains stock-confirmation, restock, and discontinuation
/// flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_name")]
pub enum CatalogIntegrationEvent {
    /// A catalog item's price changed.
    #[serde(rename = "ProductPriceChangedIntegrationEvent")]
    ProductPriceChanged(ProductPriceChangedIntegrationEventPayload),
}

impl IntegrationEvent for CatalogIntegrationEvent {
    fn event_name(&self) -> &'static str {
        match self {
            Self::ProductPriceChanged(_) => PRODUCT_PRICE_CHANGED,
        }
    }

    fn all_event_names() -> &'static [&'static str] {
        ALL_EVENT_NAMES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[derive(Debug)]
    enum TestError {
        Json(String),
        Mismatch(String),
    }

    impl core::fmt::Display for TestError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::Json(reason) => write!(f, "json: {reason}"),
                Self::Mismatch(reason) => write!(f, "mismatch: {reason}"),
            }
        }
    }

    impl core::error::Error for TestError {}

    impl From<serde_json::Error> for TestError {
        fn from(e: serde_json::Error) -> Self {
            Self::Json(e.to_string())
        }
    }

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), TestError> {
        match () {
            () if cond => Ok(()),
            () => Err(TestError::Mismatch(reason())),
        }
    }

    fn sample() -> CatalogIntegrationEvent {
        CatalogIntegrationEvent::ProductPriceChanged(
            ProductPriceChangedIntegrationEventPayload::new(
                Uuid::nil(),
                Decimal::from(20),
                Decimal::from(25),
            ),
        )
    }

    #[test]
    fn round_trip_via_json() -> Result<(), TestError> {
        let event = sample();
        let bytes = serde_json::to_vec(&event)?;
        let parsed: CatalogIntegrationEvent = serde_json::from_slice(&bytes)?;
        check(parsed == event, || format!("got {parsed:?}"))
    }

    #[test]
    fn wire_tag_matches_upstream_class_name() -> Result<(), TestError> {
        let event = sample();
        let value: serde_json::Value = serde_json::to_value(event)?;
        let tag = value
            .get("event_name")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .unwrap_or_default();
        check(tag == PRODUCT_PRICE_CHANGED, || format!("got tag {tag}"))
    }

    #[test]
    fn event_name_matches_constant() -> Result<(), TestError> {
        let event = sample();
        check(event.event_name() == PRODUCT_PRICE_CHANGED, || {
            format!("got {}", event.event_name())
        })
    }

    #[test]
    fn all_event_names_lists_only_product_price_changed() -> Result<(), TestError> {
        let names = CatalogIntegrationEvent::all_event_names();
        check(names == [PRODUCT_PRICE_CHANGED], || {
            format!("got {names:?}")
        })
    }
}
