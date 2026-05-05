//! [`BasketIntegrationEvent`] sum type and its
//! [`IntegrationEvent`] implementation.

use event_bus::IntegrationEvent;
use serde::{Deserialize, Serialize};

use crate::payload::UserCheckoutAcceptedIntegrationEventPayload;

/// Stable wire name for [`BasketIntegrationEvent::UserCheckoutAccepted`].
pub const USER_CHECKOUT_ACCEPTED: &str = "UserCheckoutAcceptedIntegrationEvent";

const ALL_EVENT_NAMES: &[&str] = &[USER_CHECKOUT_ACCEPTED];

/// Integration events emitted by the Basket bounded context.
///
/// Single variant for v1, mirroring upstream's single emission from
/// `Basket.API`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_name")]
pub enum BasketIntegrationEvent {
    /// A customer accepted their checkout; ordering should mint an
    /// order from the carried basket snapshot.
    #[serde(rename = "UserCheckoutAcceptedIntegrationEvent")]
    UserCheckoutAccepted(UserCheckoutAcceptedIntegrationEventPayload),
}

impl IntegrationEvent for BasketIntegrationEvent {
    fn event_name(&self) -> &'static str {
        match self {
            Self::UserCheckoutAccepted(_) => USER_CHECKOUT_ACCEPTED,
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

    use crate::payload::BasketSnapshotItem;

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
        if cond {
            Ok(())
        } else {
            Err(TestError::Mismatch(reason()))
        }
    }

    fn sample() -> BasketIntegrationEvent {
        BasketIntegrationEvent::UserCheckoutAccepted(
            UserCheckoutAcceptedIntegrationEventPayload::new(
                Uuid::nil(),
                Uuid::max(),
                vec![BasketSnapshotItem::new(
                    42,
                    "Hoodie".to_string(),
                    Decimal::from(20),
                    Some("hoodie.png".to_string()),
                    3,
                )],
            ),
        )
    }

    #[test]
    fn round_trip_via_json() -> Result<(), TestError> {
        let event = sample();
        let bytes = serde_json::to_vec(&event)?;
        let parsed: BasketIntegrationEvent = serde_json::from_slice(&bytes)?;
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
        check(tag == USER_CHECKOUT_ACCEPTED, || format!("got {tag}"))
    }

    #[test]
    fn event_name_matches_constant() -> Result<(), TestError> {
        let event = sample();
        check(event.event_name() == USER_CHECKOUT_ACCEPTED, || {
            format!("got {}", event.event_name())
        })
    }

    #[test]
    fn all_event_names_lists_only_user_checkout_accepted() -> Result<(), TestError> {
        let names = BasketIntegrationEvent::all_event_names();
        check(names == [USER_CHECKOUT_ACCEPTED], || {
            format!("got {names:?}")
        })
    }
}
