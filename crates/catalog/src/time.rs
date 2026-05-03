//! Conversions between [`chrono::DateTime<Utc>`] (used by the domain)
//! and [`jiff::Timestamp`] (toasty's wire type).
//!
//! Mirror of `ordering-infrastructure::time` to keep catalog
//! self-contained.  Extracting both into a shared crate is deferred
//! until a third bounded context needs the same bridge.

use chrono::{DateTime, TimeZone, Utc};
use jiff::Timestamp;

use crate::error::Error;

/// Convert a chrono UTC timestamp to a jiff timestamp.
///
/// # Errors
/// Returns [`Error::TimeConversion`] if the seconds-plus-nanos pair is
/// out of jiff's representable range.
pub fn chrono_to_jiff(dt: DateTime<Utc>) -> Result<Timestamp, Error> {
    let nanos = i32::try_from(dt.timestamp_subsec_nanos()).map_err(|e| Error::TimeConversion {
        reason: format!("subsec nanos out of i32 range: {e}"),
    })?;
    Timestamp::new(dt.timestamp(), nanos).map_err(|e| Error::TimeConversion {
        reason: e.to_string(),
    })
}

/// Convert a jiff timestamp to a chrono UTC timestamp.
///
/// # Errors
/// Returns [`Error::TimeConversion`] on out-of-range or invalid
/// component values.
pub fn jiff_to_chrono(ts: Timestamp) -> Result<DateTime<Utc>, Error> {
    let secs = ts.as_second();
    let nanos = u32::try_from(ts.subsec_nanosecond()).map_err(|e| Error::TimeConversion {
        reason: format!("subsec nanos out of u32 range: {e}"),
    })?;
    Utc.timestamp_opt(secs, nanos)
        .single()
        .ok_or_else(|| Error::TimeConversion {
            reason: "ambiguous or invalid chrono timestamp".to_string(),
        })
}
