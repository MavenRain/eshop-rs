//! Toasty row for [`WebhookSubscription`](crate::subscription::WebhookSubscription).

use jiff::Timestamp;
use uuid::Uuid;

/// One row per webhook subscription.
#[derive(Debug, toasty::Model)]
pub struct WebhookSubscriptionRow {
    /// Subscription identifier (primary key).
    #[key]
    pub id: Uuid,

    /// Wire-name of the integration event.  Indexed so the delivery
    /// worker can look up matching subscriptions when an event
    /// arrives.
    #[index]
    pub webhook_type: String,

    /// Destination URL the delivery worker POSTs to.
    pub destination_url: String,

    /// Shared secret.
    pub token: String,

    /// Identifier of the caller who registered this subscription.
    /// Indexed so the list endpoint can scope to the caller.
    #[index]
    pub grantor_id: Uuid,

    /// Creation timestamp.
    pub created_at: Timestamp,
}
