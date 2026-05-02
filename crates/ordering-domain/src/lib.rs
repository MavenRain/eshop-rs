//! Ordering bounded context: domain types for the `eshop-rs` port.
//!
//! Type-driven core of the Ordering service.  Houses aggregates, value
//! objects, domain events, and a hand-rolled `Error` enum.  Persistence and
//! transport live in sibling crates; this crate has no I/O.
