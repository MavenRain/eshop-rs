//! [`CatalogItem`] aggregate root.
//!
//! Mirrors `eShop.Catalog.API.Model.CatalogItem`.  Upstream exposes
//! `RemoveStock(int quantityDesired)` and `AddStock(int quantity)` as
//! mutating methods that throw `CatalogDomainException` on invariant
//! breaks; we model both as consuming-self functions returning
//! `Result<(Self, Units), Error>` instead, with the resulting tuple
//! carrying the updated aggregate plus the count actually
//! removed/added (so the caller can compare against what it asked for).
//!
//! Upstream's `Embedding : Pgvector.Vector?` field is **out of scope**
//! for v1 of the Rust port (see crate-level docs).  Pgvector / AI
//! semantic search will land in a follow-up dedicated commit.

use uuid::Uuid;

use crate::brand::CatalogBrandId;
use crate::error::Error;
use crate::event::{DomainEvent, ProductPriceChangedEvent};
use crate::kind::CatalogKindId;
use crate::money::Price;
use crate::stock::{MaxStockThreshold, RestockThreshold, Stock, Units};
use crate::strings::{ItemDescription, ItemName, PictureFileName};

/// Identifier for a [`CatalogItem`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CatalogItemId(Uuid);

impl CatalogItemId {
    /// Generate a fresh identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Underlying [`Uuid`].
    #[must_use]
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for CatalogItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for CatalogItemId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl From<CatalogItemId> for Uuid {
    fn from(id: CatalogItemId) -> Self {
        id.0
    }
}

/// Aggregate root for a single product in the catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogItem {
    id: CatalogItemId,
    name: ItemName,
    description: Option<ItemDescription>,
    price: Price,
    picture_file_name: Option<PictureFileName>,
    brand_id: CatalogBrandId,
    kind_id: CatalogKindId,
    available_stock: Stock,
    restock_threshold: RestockThreshold,
    max_stock_threshold: MaxStockThreshold,
    on_reorder: bool,
    domain_events: Vec<DomainEvent>,
}

impl CatalogItem {
    /// Construct a fresh catalog item.
    ///
    /// # Errors
    /// Returns [`Error::RestockExceedsMax`] if `restock_threshold` is
    /// strictly greater than `max_stock_threshold`, or
    /// [`Error::InitialStockExceedsMax`] if `available_stock` is
    /// strictly greater than `max_stock_threshold`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: CatalogItemId,
        name: ItemName,
        description: Option<ItemDescription>,
        price: Price,
        picture_file_name: Option<PictureFileName>,
        brand_id: CatalogBrandId,
        kind_id: CatalogKindId,
        available_stock: Stock,
        restock_threshold: RestockThreshold,
        max_stock_threshold: MaxStockThreshold,
    ) -> Result<Self, Error> {
        let restock = u32::from(restock_threshold);
        let max = u32::from(max_stock_threshold);
        let stock = u32::from(available_stock);
        match () {
            () if restock > max => Err(Error::RestockExceedsMax { restock, max }),
            () if stock > max => Err(Error::InitialStockExceedsMax { stock, max }),
            () => Ok(Self {
                id,
                name,
                description,
                price,
                picture_file_name,
                brand_id,
                kind_id,
                available_stock,
                restock_threshold,
                max_stock_threshold,
                on_reorder: false,
                domain_events: Vec::new(),
            }),
        }
    }

    /// Rehydrate an existing catalog item from persistence.  Bypasses
    /// invariant checks because the values are assumed already valid;
    /// the persistence layer is the only legitimate caller.
    ///
    /// # Errors
    /// Currently infallible; the `Result` return type is reserved for
    /// future invariants surfaced from the row layer.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: CatalogItemId,
        name: ItemName,
        description: Option<ItemDescription>,
        price: Price,
        picture_file_name: Option<PictureFileName>,
        brand_id: CatalogBrandId,
        kind_id: CatalogKindId,
        available_stock: Stock,
        restock_threshold: RestockThreshold,
        max_stock_threshold: MaxStockThreshold,
        on_reorder: bool,
    ) -> Result<Self, Error> {
        Ok(Self {
            id,
            name,
            description,
            price,
            picture_file_name,
            brand_id,
            kind_id,
            available_stock,
            restock_threshold,
            max_stock_threshold,
            on_reorder,
            domain_events: Vec::new(),
        })
    }

    /// Identifier.
    #[must_use]
    pub fn id(&self) -> CatalogItemId {
        self.id
    }

    /// Display name.
    #[must_use]
    pub fn name(&self) -> &ItemName {
        &self.name
    }

    /// Description, if present.
    #[must_use]
    pub fn description(&self) -> Option<&ItemDescription> {
        self.description.as_ref()
    }

    /// Per-item price.
    #[must_use]
    pub fn price(&self) -> Price {
        self.price
    }

    /// Picture file name, if present.
    #[must_use]
    pub fn picture_file_name(&self) -> Option<&PictureFileName> {
        self.picture_file_name.as_ref()
    }

    /// Brand identifier.
    #[must_use]
    pub fn brand_id(&self) -> CatalogBrandId {
        self.brand_id
    }

    /// Kind identifier.
    #[must_use]
    pub fn kind_id(&self) -> CatalogKindId {
        self.kind_id
    }

    /// Quantity currently in stock.
    #[must_use]
    pub fn available_stock(&self) -> Stock {
        self.available_stock
    }

    /// Stock level at which a reorder is triggered.
    #[must_use]
    pub fn restock_threshold(&self) -> RestockThreshold {
        self.restock_threshold
    }

    /// Maximum stock the warehouse can hold.
    #[must_use]
    pub fn max_stock_threshold(&self) -> MaxStockThreshold {
        self.max_stock_threshold
    }

    /// True if the item is currently on a reorder cycle.
    #[must_use]
    pub fn on_reorder(&self) -> bool {
        self.on_reorder
    }

    /// Pending domain events.
    #[must_use]
    pub fn domain_events(&self) -> &[DomainEvent] {
        &self.domain_events
    }

    /// Drain pending domain events from the aggregate, returning the
    /// drained aggregate plus the events ready to publish.
    #[must_use]
    pub fn take_events(self) -> (Self, Vec<DomainEvent>) {
        let events = self.domain_events;
        (
            Self {
                domain_events: Vec::new(),
                ..self
            },
            events,
        )
    }

    /// Decrement stock by up to `requested`, returning the updated
    /// aggregate plus the [`Units`] actually removed.  Mirrors
    /// upstream's `RemoveStock` semantics: caller compares the returned
    /// count against what it requested.
    ///
    /// # Errors
    /// Returns [`Error::OutOfStock`] when [`available_stock`](Self::available_stock)
    /// is already zero.  An attempt to remove zero units is a no-op and
    /// returns `(self, Units(0))` rather than an error; upstream's
    /// "quantity desired must be greater than zero" check is enforced
    /// at the API boundary, not here, because the domain answer is
    /// well-defined: zero requested means zero removed.
    pub fn remove_stock(self, requested: Units) -> Result<(Self, Units), Error> {
        if self.available_stock.is_zero() {
            Err(Error::OutOfStock {
                item: self.name.as_str().to_string(),
            })
        } else {
            let stock_n = u32::from(self.available_stock);
            let removed_n = stock_n.min(u32::from(requested));
            let new_stock_n = stock_n.checked_sub(removed_n).ok_or(Error::StockOverflow)?;
            Ok((
                Self {
                    available_stock: Stock::from(new_stock_n),
                    ..self
                },
                Units::from(removed_n),
            ))
        }
    }

    /// Increment stock by up to `quantity`, clamped at
    /// [`max_stock_threshold`](Self::max_stock_threshold).  Sets
    /// [`on_reorder`](Self::on_reorder) to false because a successful
    /// restock satisfies the reorder.  Returns the updated aggregate
    /// plus the [`Units`] actually added.
    ///
    /// # Errors
    /// Returns [`Error::StockOverflow`] only on `u32` arithmetic
    /// overflow (extremely unlikely; the type system already bounds
    /// stock at `u32::MAX`).
    pub fn add_stock(self, quantity: Units) -> Result<(Self, Units), Error> {
        let stock_n = u32::from(self.available_stock);
        let max_n = u32::from(self.max_stock_threshold);
        let qty_n = u32::from(quantity);
        let post_add = stock_n.checked_add(qty_n).ok_or(Error::StockOverflow)?;
        let new_stock_n = post_add.min(max_n);
        let added_n = new_stock_n
            .checked_sub(stock_n)
            .ok_or(Error::StockOverflow)?;
        Ok((
            Self {
                available_stock: Stock::from(new_stock_n),
                on_reorder: false,
                ..self
            },
            Units::from(added_n),
        ))
    }

    /// Update the per-item price.  Emits a
    /// [`ProductPriceChangedEvent`] if and only if the new price
    /// differs from the current price.
    #[must_use]
    pub fn change_price(self, new_price: Price) -> Self {
        if new_price == self.price {
            self
        } else {
            let event = DomainEvent::ProductPriceChanged(ProductPriceChangedEvent::new(
                self.id, new_price, self.price,
            ));
            let domain_events = {
                let prior = self.domain_events;
                let mut next = Vec::with_capacity(prior.len() + 1);
                next.extend(prior);
                next.push(event);
                next
            };
            Self {
                price: new_price,
                domain_events,
                ..self
            }
        }
    }

    /// Mark the item as on reorder.
    #[must_use]
    pub fn mark_on_reorder(self) -> Self {
        Self {
            on_reorder: true,
            ..self
        }
    }

    /// Apply a wholesale update from an admin endpoint.  Validates the
    /// stock/threshold invariants and emits a
    /// [`ProductPriceChangedEvent`] iff `price` differs from the
    /// current price.  `on_reorder` is preserved (a wholesale update
    /// is metadata-driven, not a restock).
    ///
    /// # Errors
    /// - [`Error::RestockExceedsMax`] if `restock_threshold` is
    ///   strictly greater than `max_stock_threshold`.
    /// - [`Error::InitialStockExceedsMax`] if `available_stock` is
    ///   strictly greater than `max_stock_threshold`.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_update(
        self,
        name: ItemName,
        description: Option<ItemDescription>,
        price: Price,
        picture_file_name: Option<PictureFileName>,
        brand_id: CatalogBrandId,
        kind_id: CatalogKindId,
        available_stock: Stock,
        restock_threshold: RestockThreshold,
        max_stock_threshold: MaxStockThreshold,
    ) -> Result<Self, Error> {
        let restock_n = u32::from(restock_threshold);
        let max_n = u32::from(max_stock_threshold);
        let stock_n = u32::from(available_stock);
        match () {
            () if restock_n > max_n => Err(Error::RestockExceedsMax {
                restock: restock_n,
                max: max_n,
            }),
            () if stock_n > max_n => Err(Error::InitialStockExceedsMax {
                stock: stock_n,
                max: max_n,
            }),
            () => {
                let domain_events = if price == self.price {
                    self.domain_events
                } else {
                    let event = DomainEvent::ProductPriceChanged(ProductPriceChangedEvent::new(
                        self.id, price, self.price,
                    ));
                    let prior = self.domain_events;
                    let mut next = Vec::with_capacity(prior.len() + 1);
                    next.extend(prior);
                    next.push(event);
                    next
                };
                Ok(Self {
                    id: self.id,
                    name,
                    description,
                    price,
                    picture_file_name,
                    brand_id,
                    kind_id,
                    available_stock,
                    restock_threshold,
                    max_stock_threshold,
                    on_reorder: self.on_reorder,
                    domain_events,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::InvariantViolated { reason: reason() })
        }
    }

    fn sample_item(stock: u32, max: u32) -> Result<CatalogItem, Error> {
        CatalogItem::new(
            CatalogItemId::new(),
            ItemName::try_from(".NET Bot Black Hoodie")?,
            None,
            Price::new(Decimal::from(20))?,
            None,
            CatalogBrandId::new(),
            CatalogKindId::new(),
            Stock::from(stock),
            RestockThreshold::from(0),
            MaxStockThreshold::from(max),
        )
    }

    #[test]
    fn new_rejects_initial_stock_exceeding_max() -> Result<(), Error> {
        let outcome = sample_item(101, 100);
        check(
            matches!(
                outcome,
                Err(Error::InitialStockExceedsMax {
                    stock: 101,
                    max: 100,
                })
            ),
            || format!("expected InitialStockExceedsMax, got {outcome:?}"),
        )
    }

    #[test]
    fn new_rejects_restock_exceeding_max() -> Result<(), Error> {
        let outcome = CatalogItem::new(
            CatalogItemId::new(),
            ItemName::try_from(".NET Bot Black Hoodie")?,
            None,
            Price::new(Decimal::from(20))?,
            None,
            CatalogBrandId::new(),
            CatalogKindId::new(),
            Stock::from(0),
            RestockThreshold::from(50),
            MaxStockThreshold::from(10),
        );
        check(
            matches!(
                outcome,
                Err(Error::RestockExceedsMax {
                    restock: 50,
                    max: 10,
                })
            ),
            || format!("expected RestockExceedsMax, got {outcome:?}"),
        )
    }

    #[test]
    fn remove_stock_returns_min_of_request_and_available() -> Result<(), Error> {
        let item = sample_item(10, 100)?;
        let (next, removed) = item.remove_stock(Units::from(3))?;
        check(removed.get() == 3, || format!("removed {}", removed.get()))?;
        check(next.available_stock().get() == 7, || {
            format!("stock {}", next.available_stock().get())
        })
    }

    #[test]
    fn remove_stock_clamps_to_available() -> Result<(), Error> {
        let item = sample_item(2, 100)?;
        let (next, removed) = item.remove_stock(Units::from(10))?;
        check(removed.get() == 2, || format!("removed {}", removed.get()))?;
        check(next.available_stock().is_zero(), || {
            format!("stock {}", next.available_stock().get())
        })
    }

    #[test]
    fn remove_stock_at_zero_errors() -> Result<(), Error> {
        let item = sample_item(0, 100)?;
        let outcome = item.remove_stock(Units::from(1));
        check(matches!(outcome, Err(Error::OutOfStock { .. })), || {
            format!("expected OutOfStock, got {outcome:?}")
        })
    }

    #[test]
    fn add_stock_increments_to_max() -> Result<(), Error> {
        let item = sample_item(95, 100)?.mark_on_reorder();
        let (next, added) = item.add_stock(Units::from(20))?;
        check(added.get() == 5, || format!("added {}", added.get()))?;
        check(next.available_stock().get() == 100, || {
            format!("stock {}", next.available_stock().get())
        })?;
        check(!next.on_reorder(), || "on_reorder still set".to_string())
    }

    #[test]
    fn add_stock_under_max_increments_full_amount() -> Result<(), Error> {
        let item = sample_item(5, 100)?;
        let (next, added) = item.add_stock(Units::from(10))?;
        check(added.get() == 10, || format!("added {}", added.get()))?;
        check(next.available_stock().get() == 15, || {
            format!("stock {}", next.available_stock().get())
        })
    }

    #[test]
    fn change_price_emits_event_on_difference() -> Result<(), Error> {
        let item = sample_item(10, 100)?;
        let new_price = Price::new(Decimal::from(25))?;
        let updated = item.change_price(new_price);
        check(updated.price() == new_price, || {
            "price not updated".to_string()
        })?;
        check(updated.domain_events().len() == 1, || {
            format!("expected 1 event, got {}", updated.domain_events().len())
        })?;
        let raised = updated
            .domain_events()
            .first()
            .ok_or_else(|| Error::InvariantViolated {
                reason: "missing event".to_string(),
            })?;
        match raised {
            DomainEvent::ProductPriceChanged(e) => {
                check(e.new_price() == new_price, || {
                    "new_price mismatch".to_string()
                })?;
                check(e.old_price() == Price::new(Decimal::from(20))?, || {
                    "old_price mismatch".to_string()
                })
            }
        }
    }

    #[test]
    fn change_price_no_event_on_same_price() -> Result<(), Error> {
        let item = sample_item(10, 100)?;
        let same_price = item.price();
        let updated = item.change_price(same_price);
        check(updated.domain_events().is_empty(), || {
            format!("expected 0 events, got {}", updated.domain_events().len())
        })
    }

    #[test]
    fn take_events_drains_aggregate() -> Result<(), Error> {
        let item = sample_item(10, 100)?.change_price(Price::new(Decimal::from(25))?);
        let (drained, events) = item.take_events();
        check(events.len() == 1, || format!("events {}", events.len()))?;
        check(drained.domain_events().is_empty(), || {
            "events not drained".to_string()
        })
    }

    #[test]
    fn apply_update_emits_event_when_price_changes() -> Result<(), Error> {
        let item = sample_item(10, 100)?;
        let new_price = Price::new(Decimal::from(99))?;
        let updated = item.apply_update(
            ItemName::try_from("Updated Hoodie")?,
            Some(ItemDescription::try_from("Updated description")?),
            new_price,
            None,
            CatalogBrandId::new(),
            CatalogKindId::new(),
            Stock::from(50),
            RestockThreshold::from(10),
            MaxStockThreshold::from(150),
        )?;
        check(updated.price() == new_price, || {
            "price not updated".to_string()
        })?;
        check(updated.domain_events().len() == 1, || {
            format!("expected 1 event, got {}", updated.domain_events().len())
        })?;
        check(updated.name().as_str() == "Updated Hoodie", || {
            "name not updated".to_string()
        })
    }

    #[test]
    fn apply_update_no_event_when_price_unchanged() -> Result<(), Error> {
        let item = sample_item(10, 100)?;
        let same_price = item.price();
        let updated = item.apply_update(
            ItemName::try_from("Renamed")?,
            None,
            same_price,
            None,
            CatalogBrandId::new(),
            CatalogKindId::new(),
            Stock::from(10),
            RestockThreshold::from(0),
            MaxStockThreshold::from(100),
        )?;
        check(updated.domain_events().is_empty(), || {
            format!("expected 0 events, got {}", updated.domain_events().len())
        })
    }

    #[test]
    fn apply_update_rejects_invalid_thresholds() -> Result<(), Error> {
        let item = sample_item(10, 100)?;
        let outcome = item.apply_update(
            ItemName::try_from("X")?,
            None,
            Price::new(Decimal::from(20))?,
            None,
            CatalogBrandId::new(),
            CatalogKindId::new(),
            Stock::from(0),
            RestockThreshold::from(50),
            MaxStockThreshold::from(10),
        );
        check(
            matches!(
                outcome,
                Err(Error::RestockExceedsMax {
                    restock: 50,
                    max: 10
                })
            ),
            || format!("expected RestockExceedsMax, got {outcome:?}"),
        )
    }
}
