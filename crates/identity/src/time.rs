//! Conversions between [`chrono::DateTime<Utc>`] and [`jiff::Timestamp`].

use chrono::{DateTime, TimeZone, Utc};
use jiff::Timestamp;

use crate::error::Error;

/// Convert a chrono UTC timestamp to a jiff timestamp.
///
/// # Errors
/// Returns [`Error::TimeConversion`] if the seconds-plus-nanos pair
/// is out of jiff's representable range.
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
/// Returns [`Error::TimeConversion`] if the seconds part is out of
/// chrono's representable range.
pub fn jiff_to_chrono(ts: Timestamp) -> Result<DateTime<Utc>, Error> {
    Utc.timestamp_opt(ts.as_second(), ts.subsec_nanosecond().unsigned_abs())
        .single()
        .ok_or_else(|| Error::TimeConversion {
            reason: format!("seconds {} not representable", ts.as_second()),
        })
}
