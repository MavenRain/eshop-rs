//! Persistence boundary for the
//! [`WebhookSubscription`](crate::subscription::WebhookSubscription)
//! aggregate.

use uuid::Uuid;

use crate::error::Error;
use crate::mapper::{row_to_subscription, subscription_to_row};
use crate::row::WebhookSubscriptionRow;
use crate::subscription::{WebhookSubscription, WebhookSubscriptionId};

/// Persistence boundary for [`WebhookSubscription`].
pub struct WebhookSubscriptionRepository;

impl WebhookSubscriptionRepository {
    /// Insert a new subscription.
    ///
    /// # Errors
    /// Propagates toasty and mapping errors.
    pub async fn add<E>(executor: &mut E, subscription: &WebhookSubscription) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let row = subscription_to_row(subscription)?;
        toasty::create!(WebhookSubscriptionRow {
            id: row.id,
            webhook_type: row.webhook_type,
            destination_url: row.destination_url,
            token: row.token,
            grantor_id: row.grantor_id,
            created_at: row.created_at,
        })
        .exec(executor)
        .await?;
        Ok(())
    }

    /// Load a subscription by id.
    ///
    /// # Errors
    /// Propagates toasty errors (including not-found) and mapping
    /// errors.
    pub async fn get_by_id<E>(
        executor: &mut E,
        id: WebhookSubscriptionId,
    ) -> Result<WebhookSubscription, Error>
    where
        E: toasty::Executor,
    {
        let row = WebhookSubscriptionRow::get_by_id(executor, &id.into_uuid()).await?;
        row_to_subscription(&row)
    }

    /// Load every subscription owned by `grantor_id`.
    ///
    /// # Errors
    /// Propagates toasty errors and mapping errors.
    pub async fn list_by_grantor<E>(
        executor: &mut E,
        grantor_id: Uuid,
    ) -> Result<Vec<WebhookSubscription>, Error>
    where
        E: toasty::Executor,
    {
        let rows: Vec<WebhookSubscriptionRow> = toasty::query!(
            WebhookSubscriptionRow FILTER .grantor_id == #grantor_id
        )
        .exec(executor)
        .await?;
        rows.iter().map(row_to_subscription).collect()
    }

    /// Load every subscription that matches `webhook_type` (used by
    /// the future delivery worker).
    ///
    /// # Errors
    /// Propagates toasty errors and mapping errors.
    pub async fn list_by_type<E>(
        executor: &mut E,
        webhook_type: &str,
    ) -> Result<Vec<WebhookSubscription>, Error>
    where
        E: toasty::Executor,
    {
        let needle = webhook_type.to_string();
        let rows: Vec<WebhookSubscriptionRow> = toasty::query!(
            WebhookSubscriptionRow FILTER .webhook_type == #needle
        )
        .exec(executor)
        .await?;
        rows.iter().map(row_to_subscription).collect()
    }

    /// Delete a subscription by id.  The two-query pattern (load then
    /// delete) is the same toasty FFI shape catalog uses.
    ///
    /// # Errors
    /// Propagates toasty errors (including not-found).
    pub async fn delete_by_id<E>(executor: &mut E, id: WebhookSubscriptionId) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let row = WebhookSubscriptionRow::get_by_id(executor, &id.into_uuid()).await?;
        row.delete().exec(executor).await?;
        Ok(())
    }
}
