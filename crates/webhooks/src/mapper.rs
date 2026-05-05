//! Bidirectional mappers between [`WebhookSubscription`] and
//! [`WebhookSubscriptionRow`].

use uuid::Uuid;

use crate::error::Error;
use crate::row::WebhookSubscriptionRow;
use crate::strings::{WebhookToken, WebhookType, WebhookUrl};
use crate::subscription::{WebhookSubscription, WebhookSubscriptionId};
use crate::time::{chrono_to_jiff, jiff_to_chrono};

/// Project a [`WebhookSubscription`] into a row helper ready for
/// insert.
///
/// # Errors
/// Returns [`Error::TimeConversion`] if the creation timestamp is
/// outside jiff's representable range.
pub fn subscription_to_row(
    subscription: &WebhookSubscription,
) -> Result<NewWebhookSubscriptionRow, Error> {
    Ok(NewWebhookSubscriptionRow {
        id: subscription.id().into_uuid(),
        webhook_type: subscription.webhook_type().as_str().to_string(),
        destination_url: subscription.destination_url().as_str().to_string(),
        token: subscription.token().as_str().to_string(),
        grantor_id: subscription.grantor_id(),
        created_at: chrono_to_jiff(subscription.created_at())?,
    })
}

/// Rehydrate a [`WebhookSubscription`] from a loaded row.
///
/// # Errors
/// - [`Error::TimeConversion`] if the persisted timestamp is out of
///   chrono's representable range.
/// - [`Error::EmptyString`] if a non-empty newtype was persisted as
///   blank (would indicate a database integrity break).
pub fn row_to_subscription(row: &WebhookSubscriptionRow) -> Result<WebhookSubscription, Error> {
    Ok(WebhookSubscription::new(
        WebhookSubscriptionId::from(row.id),
        WebhookType::try_from(row.webhook_type.clone())?,
        WebhookUrl::try_from(row.destination_url.clone())?,
        WebhookToken::try_from(row.token.clone())?,
        row.grantor_id,
        jiff_to_chrono(row.created_at)?,
    ))
}

/// Bag-of-columns helper for inserting a [`WebhookSubscriptionRow`].
#[derive(Debug, Clone)]
pub struct NewWebhookSubscriptionRow {
    pub id: Uuid,
    pub webhook_type: String,
    pub destination_url: String,
    pub token: String,
    pub grantor_id: Uuid,
    pub created_at: jiff::Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Utc;

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
    fn round_trip_via_row() -> Result<(), Error> {
        let subscription = sample_subscription()?;
        let helper = subscription_to_row(&subscription)?;
        let row = WebhookSubscriptionRow {
            id: helper.id,
            webhook_type: helper.webhook_type,
            destination_url: helper.destination_url,
            token: helper.token,
            grantor_id: helper.grantor_id,
            created_at: helper.created_at,
        };
        let restored = row_to_subscription(&row)?;
        check(restored.id() == subscription.id(), || {
            "id mismatch".to_string()
        })?;
        check(
            restored.webhook_type() == subscription.webhook_type(),
            || "type mismatch".to_string(),
        )?;
        check(
            restored.destination_url() == subscription.destination_url(),
            || "url mismatch".to_string(),
        )?;
        check(restored.token() == subscription.token(), || {
            "token mismatch".to_string()
        })?;
        check(restored.grantor_id() == subscription.grantor_id(), || {
            "grantor mismatch".to_string()
        })
    }

    #[test]
    fn row_to_subscription_rejects_empty_type() -> Result<(), Error> {
        let row = WebhookSubscriptionRow {
            id: Uuid::new_v4(),
            webhook_type: String::new(),
            destination_url: "https://example.test/hook".to_string(),
            token: "secret".to_string(),
            grantor_id: Uuid::new_v4(),
            created_at: chrono_to_jiff(Utc::now())?,
        };
        let outcome = row_to_subscription(&row);
        check(
            matches!(
                outcome,
                Err(Error::EmptyString {
                    field: "webhook type"
                })
            ),
            || format!("got {outcome:?}"),
        )
    }
}
