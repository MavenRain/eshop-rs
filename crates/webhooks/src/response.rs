//! Response DTOs serialized to JSON response bodies.

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::subscription::WebhookSubscription;

/// `GET /api/webhooks/{id}` and the elements of `GET /api/webhooks`
/// response bodies.
#[derive(Debug, Clone, Serialize)]
pub struct WebhookSubscriptionResponse {
    id: Uuid,
    webhook_type: String,
    destination_url: String,
    grantor_id: Uuid,
    created_at: DateTime<Utc>,
}

impl From<&WebhookSubscription> for WebhookSubscriptionResponse {
    fn from(subscription: &WebhookSubscription) -> Self {
        Self {
            id: subscription.id().into_uuid(),
            webhook_type: subscription.webhook_type().as_str().to_string(),
            destination_url: subscription.destination_url().as_str().to_string(),
            grantor_id: subscription.grantor_id(),
            created_at: subscription.created_at(),
        }
    }
}

/// `POST /api/webhooks` response body.
#[derive(Debug, Clone, Serialize)]
pub struct CreatedSubscriptionResponse {
    id: Uuid,
}

impl CreatedSubscriptionResponse {
    /// Wrap a fresh subscription id for the JSON response.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::error::Error;
    use crate::strings::{WebhookToken, WebhookType, WebhookUrl};
    use crate::subscription::WebhookSubscriptionId;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    fn sample_subscription() -> Result<WebhookSubscription, Error> {
        Ok(WebhookSubscription::new(
            WebhookSubscriptionId::new(),
            WebhookType::try_from("OrderShippedIntegrationEvent")?,
            WebhookUrl::try_from("https://example.test/hook")?,
            WebhookToken::try_from("secret")?,
            Uuid::new_v4(),
            Utc::now(),
        ))
    }

    #[test]
    fn response_omits_token() -> Result<(), Error> {
        let subscription = sample_subscription()?;
        let response = WebhookSubscriptionResponse::from(&subscription);
        let body = serde_json::to_string(&response)?;
        check(!body.contains("secret"), || {
            format!("token leaked into response: {body}")
        })
    }

    #[test]
    fn response_carries_grantor_id() -> Result<(), Error> {
        let subscription = sample_subscription()?;
        let response = WebhookSubscriptionResponse::from(&subscription);
        check(response.grantor_id == subscription.grantor_id(), || {
            "grantor mismatch".to_string()
        })
    }
}
