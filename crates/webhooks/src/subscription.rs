//! [`WebhookSubscription`] aggregate.
//!
//! Mirrors upstream eShop's `Webhooks.API.Model.WebhookSubscription`:
//! per-grantor record of a destination URL + token to call when an
//! integration event of [`WebhookType`] fires.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::strings::{WebhookToken, WebhookType, WebhookUrl};

/// Identifier for a [`WebhookSubscription`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct WebhookSubscriptionId(Uuid);

impl WebhookSubscriptionId {
    /// Generate a fresh random identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Borrow as the underlying [`Uuid`].
    #[must_use]
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for WebhookSubscriptionId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for WebhookSubscriptionId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<WebhookSubscriptionId> for Uuid {
    fn from(id: WebhookSubscriptionId) -> Self {
        id.0
    }
}

/// One subscription record.  Each grantor (the authenticated caller
/// of the registration endpoint) may hold many of these, one per
/// (event type × destination) pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebhookSubscription {
    id: WebhookSubscriptionId,
    webhook_type: WebhookType,
    destination_url: WebhookUrl,
    token: WebhookToken,
    grantor_id: Uuid,
    created_at: DateTime<Utc>,
}

impl WebhookSubscription {
    /// Construct.
    #[must_use]
    pub fn new(
        id: WebhookSubscriptionId,
        webhook_type: WebhookType,
        destination_url: WebhookUrl,
        token: WebhookToken,
        grantor_id: Uuid,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            webhook_type,
            destination_url,
            token,
            grantor_id,
            created_at,
        }
    }

    /// Subscription identifier.
    #[must_use]
    pub fn id(&self) -> WebhookSubscriptionId {
        self.id
    }

    /// Wire name of the integration event this subscription matches.
    #[must_use]
    pub fn webhook_type(&self) -> &WebhookType {
        &self.webhook_type
    }

    /// Destination URL.
    #[must_use]
    pub fn destination_url(&self) -> &WebhookUrl {
        &self.destination_url
    }

    /// Shared secret.
    #[must_use]
    pub fn token(&self) -> &WebhookToken {
        &self.token
    }

    /// Grantor (the caller who registered this subscription).
    #[must_use]
    pub fn grantor_id(&self) -> Uuid {
        self.grantor_id
    }

    /// Creation timestamp (UTC).
    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
