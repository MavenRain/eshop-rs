//! Request DTOs deserialized from JSON bodies and query strings.

use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::Error;
use crate::strings::{WebhookToken, WebhookType, WebhookUrl};
use crate::subscription::{WebhookSubscription, WebhookSubscriptionId};

/// `POST /api/webhooks` body.
///
/// `grantor_id` is caller-supplied for v1.  Once an identity bounded
/// context lands, it will come from the authenticated principal
/// instead and will be removed from the request shape.
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterSubscriptionRequest {
    webhook_type: String,
    destination_url: String,
    token: String,
    grantor_id: Uuid,
}

impl RegisterSubscriptionRequest {
    /// Project this request into a fresh [`WebhookSubscription`].
    ///
    /// # Errors
    /// Returns [`Error::EmptyString`] for blank fields.
    pub fn try_into_subscription(self) -> Result<WebhookSubscription, Error> {
        Ok(WebhookSubscription::new(
            WebhookSubscriptionId::new(),
            WebhookType::try_from(self.webhook_type)?,
            WebhookUrl::try_from(self.destination_url)?,
            WebhookToken::try_from(self.token)?,
            self.grantor_id,
            Utc::now(),
        ))
    }
}

/// `GET /api/webhooks?grantor_id=...` query string.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ListSubscriptionsQuery {
    grantor_id: Uuid,
}

impl ListSubscriptionsQuery {
    /// Grantor whose subscriptions the caller wants.
    #[must_use]
    pub fn grantor_id(&self) -> Uuid {
        self.grantor_id
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
            grantor_id: Uuid::new_v4(),
        }
    }

    #[test]
    fn register_projects_to_subscription() -> Result<(), Error> {
        let request = valid_register_request();
        let grantor = request.grantor_id;
        let subscription = request.try_into_subscription()?;
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
        let outcome = request.try_into_subscription();
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
        let outcome = request.try_into_subscription();
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
