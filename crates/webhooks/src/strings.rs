//! Non-empty string newtypes for webhook subscription payload fields.

use crate::error::Error;

macro_rules! non_empty_string_newtype {
    ($(#[$meta:meta])* $name:ident, $field:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            /// Borrow the inner string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;

            fn try_from(s: String) -> Result<Self, Error> {
                if s.trim().is_empty() {
                    Err(Error::EmptyString { field: $field })
                } else {
                    Ok(Self(s))
                }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = Error;

            fn try_from(s: &str) -> Result<Self, Error> {
                Self::try_from(s.to_string())
            }
        }

        impl From<$name> for String {
            fn from(v: $name) -> Self {
                v.0
            }
        }
    };
}

non_empty_string_newtype!(
    /// Wire-name of an integration event the subscription cares
    /// about.  Matches the
    /// [`IntegrationEvent::event_name`](event_bus::IntegrationEvent::event_name)
    /// returned by the publisher (e.g.
    /// `OrderShippedIntegrationEvent`).
    WebhookType,
    "webhook type"
);

non_empty_string_newtype!(
    /// Destination URL the delivery worker POSTs to when a matching
    /// event fires.  Validated as non-empty here; deeper URL parsing
    /// happens at the delivery boundary so the subscription API
    /// stays permissive on storage shape.
    WebhookUrl,
    "webhook url"
);

non_empty_string_newtype!(
    /// Shared secret the subscriber's endpoint expects in the
    /// delivery's authorization header (or query string).
    WebhookToken,
    "webhook token"
);

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

    #[test]
    fn empty_webhook_type_rejected() -> Result<(), Error> {
        let outcome = WebhookType::try_from(String::new());
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

    #[test]
    fn whitespace_webhook_url_rejected() -> Result<(), Error> {
        let outcome = WebhookUrl::try_from("   ");
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

    #[test]
    fn non_empty_webhook_token_accepted() -> Result<(), Error> {
        let token = WebhookToken::try_from("abc123")?;
        check(token.as_str() == "abc123", || {
            format!("got {}", token.as_str())
        })
    }
}
