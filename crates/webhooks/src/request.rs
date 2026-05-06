//! Request DTOs deserialized from JSON bodies.
//!
//! `grantor_id` is no longer in the request shape; it comes from
//! the authenticated principal at the handler boundary.  The list
//! endpoint similarly takes no query string — it lists subscriptions
//! owned by the authenticated caller.

use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::Error;
use crate::strings::{WebhookToken, WebhookType, WebhookUrl};
use crate::subscription::{WebhookSubscription, WebhookSubscriptionId};

/// `POST /api/webhooks` body.
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterSubscriptionRequest {
    webhook_type: String,
    destination_url: String,
    token: String,
}

impl RegisterSubscriptionRequest {
    /// Project this request into a fresh [`WebhookSubscription`]
    /// owned by `grantor_id` (supplied by the handler from the
    /// authenticated principal).
    ///
    /// # Errors
    /// Returns [`Error::EmptyString`] for blank fields.
    pub fn try_into_subscription(self, grantor_id: Uuid) -> Result<WebhookSubscription, Error> {
        Ok(WebhookSubscription::new(
            WebhookSubscriptionId::new(),
            WebhookType::try_from(self.webhook_type)?,
            WebhookUrl::try_from(self.destination_url)?,
            WebhookToken::try_from(self.token)?,
            grantor_id,
            Utc::now(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    fn valid_register_request() -> RegisterSubscriptionRequest {
        RegisterSubscriptionRequest {
            webhook_type: "OrderShippedIntegrationEvent".to_string(),
            destination_url: "https://example.test/hook".to_string(),
            token: "secret".to_string(),
        }
    }

    #[test]
    fn register_projects_to_subscription() -> Result<(), Error> {
        let grantor = Uuid::new_v4();
        let subscription = valid_register_request().try_into_subscription(grantor)?;
        check(
            subscription.webhook_type().as_str() == "OrderShippedIntegrationEvent",
            || format!("type {}", subscription.webhook_type().as_str()),
        )?;
        check(subscription.grantor_id() == grantor, || {
            "grantor mismatch".to_string()
        })
    }

    #[test]
    fn register_rejects_empty_token() -> Result<(), Error> {
        let request = RegisterSubscriptionRequest {
            token: String::new(),
            ..valid_register_request()
        };
        let outcome = request.try_into_subscription(Uuid::new_v4());
        check(
            matches!(
                outcome,
                Err(Error::EmptyString {
                    field: "webhook token"
                })
            ),
            || format!("got {outcome:?}"),
        )
    }

    #[test]
    fn register_rejects_empty_url() -> Result<(), Error> {
        let request = RegisterSubscriptionRequest {
            destination_url: String::new(),
            ..valid_register_request()
        };
        let outcome = request.try_into_subscription(Uuid::new_v4());
        check(
            matches!(
                outcome,
                Err(Error::EmptyString {
                    field: "webhook url"
                })
            ),
            || format!("got {outcome:?}"),
        )
    }
}
