//! Bidirectional mappers between catalog domain aggregates and database rows.
//!
//! Domain types preserve their invariants (private fields, validated
//! constructors); rows are pure bags of columns.  These functions are
//! the single boundary where the two representations meet.
//!
//! `New*Row` types are crate-internal bag-of-columns helpers, identical
//! in spirit to the `NewOrderRow` / `NewBuyerRow` precedents in
//! `ordering-infrastructure::mapper`: callers vary how they flush
//! (single insert vs. transaction batch) without re-binding the toasty
//! row directly.

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::brand::{CatalogBrand, CatalogBrandId};
use crate::error::Error;
use crate::item::{CatalogItem, CatalogItemId};
use crate::kind::{CatalogKind, CatalogKindId};
use crate::money::Price;
use crate::row::{CatalogBrandRow, CatalogItemRow, CatalogKindRow};
use crate::stock::{MaxStockThreshold, RestockThreshold, Stock};
use crate::strings::{BrandName, ItemDescription, ItemName, KindName, PictureFileName};

/// Project a [`CatalogItem`] into a [`NewCatalogItemRow`] ready for
/// insert or update.
///
/// Infallible: every column maps directly from a domain accessor with
/// no checked conversion.  The corresponding `order_to_rows` /
/// `buyer_to_rows` mappers in `ordering-infrastructure` are fallible
/// only because they call `chrono_to_jiff`, which catalog rows do not
/// need.
#[must_use]
pub fn item_to_row(item: &CatalogItem) -> NewCatalogItemRow {
    NewCatalogItemRow {
        id: item.id().into_i32(),
        name: item.name().as_str().to_string(),
        description: item.description().map(|d| d.as_str().to_string()),
        price: item.price().into_decimal(),
        picture_file_name: item.picture_file_name().map(|p| p.as_str().to_string()),
        brand_id: item.brand_id().into_uuid(),
        kind_id: item.kind_id().into_uuid(),
        available_stock: i64::from(item.available_stock().get()),
        restock_threshold: i64::from(item.restock_threshold().get()),
        max_stock_threshold: i64::from(item.max_stock_threshold().get()),
        on_reorder: item.on_reorder(),
    }
}

/// Rehydrate a [`CatalogItem`] from a loaded row.
///
/// # Errors
/// - [`Error::NumericRange`] if any of the three stock columns is
///   outside `u32` range.
/// - [`Error::EmptyString`] / [`Error::NegativePrice`] if a persisted
///   value violates a domain invariant.
pub fn row_to_item(row: &CatalogItemRow) -> Result<CatalogItem, Error> {
    let stock_u32 = u32::try_from(row.available_stock).map_err(|_| Error::NumericRange {
        context: "catalog_items.available_stock".to_string(),
    })?;
    let restock_u32 = u32::try_from(row.restock_threshold).map_err(|_| Error::NumericRange {
        context: "catalog_items.restock_threshold".to_string(),
    })?;
    let max_u32 = u32::try_from(row.max_stock_threshold).map_err(|_| Error::NumericRange {
        context: "catalog_items.max_stock_threshold".to_string(),
    })?;
    let description = row
        .description
        .as_ref()
        .map(|s| ItemDescription::try_from(s.clone()))
        .transpose()?;
    let picture_file_name = row
        .picture_file_name
        .as_ref()
        .map(|s| PictureFileName::try_from(s.clone()))
        .transpose()?;
    CatalogItem::rehydrate(
        CatalogItemId::from(row.id),
        ItemName::try_from(row.name.clone())?,
        description,
        Price::new(row.price)?,
        picture_file_name,
        CatalogBrandId::from(row.brand_id),
        CatalogKindId::from(row.kind_id),
        Stock::from(stock_u32),
        RestockThreshold::from(restock_u32),
        MaxStockThreshold::from(max_u32),
        row.on_reorder,
    )
}

/// Project a [`CatalogBrand`] into a [`NewCatalogBrandRow`].
#[must_use]
pub fn brand_to_row(brand: &CatalogBrand) -> NewCatalogBrandRow {
    NewCatalogBrandRow {
        id: brand.id().into_uuid(),
        name: brand.name().as_str().to_string(),
    }
}

/// Rehydrate a [`CatalogBrand`] from a loaded row.
///
/// # Errors
/// [`Error::EmptyString`] if the persisted `name` is empty (would
/// indicate a database integrity break, since [`BrandName`] cannot be
/// constructed empty).
pub fn row_to_brand(row: &CatalogBrandRow) -> Result<CatalogBrand, Error> {
    Ok(CatalogBrand::new(
        CatalogBrandId::from(row.id),
        BrandName::try_from(row.name.clone())?,
    ))
}

/// Project a [`CatalogKind`] into a [`NewCatalogKindRow`].
#[must_use]
pub fn kind_to_row(kind: &CatalogKind) -> NewCatalogKindRow {
    NewCatalogKindRow {
        id: kind.id().into_uuid(),
        name: kind.name().as_str().to_string(),
    }
}

/// Rehydrate a [`CatalogKind`] from a loaded row.
///
/// # Errors
/// [`Error::EmptyString`] if the persisted `name` is empty.
pub fn row_to_kind(row: &CatalogKindRow) -> Result<CatalogKind, Error> {
    Ok(CatalogKind::new(
        CatalogKindId::from(row.id),
        KindName::try_from(row.name.clone())?,
    ))
}

/// Bag-of-columns representation of [`CatalogItem`] for insert/update.
#[derive(Debug, Clone)]
pub struct NewCatalogItemRow {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub picture_file_name: Option<String>,
    pub brand_id: Uuid,
    pub kind_id: Uuid,
    pub available_stock: i64,
    pub restock_threshold: i64,
    pub max_stock_threshold: i64,
    pub on_reorder: bool,
}

#[derive(Debug, Clone)]
pub struct NewCatalogBrandRow {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct NewCatalogKindRow {
    pub id: Uuid,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::InvariantViolated { reason: reason() })
        }
    }

    fn sample_item() -> Result<CatalogItem, Error> {
        CatalogItem::new(
            CatalogItemId::from(1),
            ItemName::try_from(".NET Bot Black Hoodie")?,
            Some(ItemDescription::try_from("Stylish hoodie")?),
            Price::new(Decimal::from(20))?,
            Some(PictureFileName::try_from("hoodie.png")?),
            CatalogBrandId::new(),
            CatalogKindId::new(),
            Stock::from(100),
            RestockThreshold::from(20),
            MaxStockThreshold::from(200),
        )
    }

    #[test]
    fn item_to_row_projects_optional_columns() -> Result<(), Error> {
        let item = sample_item()?;
        let row = item_to_row(&item);
        check(row.id == item.id().into_i32(), || "id mismatch".to_string())?;
        check(row.name == ".NET Bot Black Hoodie", || {
            format!("name {}", row.name)
        })?;
        check(row.description.as_deref() == Some("Stylish hoodie"), || {
            format!("description {:?}", row.description)
        })?;
        check(
            row.picture_file_name.as_deref() == Some("hoodie.png"),
            || format!("picture {:?}", row.picture_file_name),
        )?;
        check(row.available_stock == 100, || {
            format!("stock {}", row.available_stock)
        })
    }

    #[test]
    fn item_round_trip_via_row() -> Result<(), Error> {
        let item = sample_item()?;
        let row_helper = item_to_row(&item);
        let row = CatalogItemRow {
            id: row_helper.id,
            name: row_helper.name,
            description: row_helper.description,
            price: row_helper.price,
            picture_file_name: row_helper.picture_file_name,
            brand_id: row_helper.brand_id,
            kind_id: row_helper.kind_id,
            available_stock: row_helper.available_stock,
            restock_threshold: row_helper.restock_threshold,
            max_stock_threshold: row_helper.max_stock_threshold,
            on_reorder: row_helper.on_reorder,
        };
        let restored = row_to_item(&row)?;
        check(restored.id() == item.id(), || "id mismatch".to_string())?;
        check(restored.name() == item.name(), || {
            "name mismatch".to_string()
        })?;
        check(restored.description() == item.description(), || {
            "description mismatch".to_string()
        })?;
        check(restored.price() == item.price(), || {
            "price mismatch".to_string()
        })?;
        check(
            restored.picture_file_name() == item.picture_file_name(),
            || "picture_file_name mismatch".to_string(),
        )?;
        check(restored.brand_id() == item.brand_id(), || {
            "brand_id mismatch".to_string()
        })?;
        check(restored.kind_id() == item.kind_id(), || {
            "kind_id mismatch".to_string()
        })?;
        check(restored.available_stock() == item.available_stock(), || {
            "stock mismatch".to_string()
        })?;
        check(
            restored.restock_threshold() == item.restock_threshold(),
            || "restock mismatch".to_string(),
        )?;
        check(
            restored.max_stock_threshold() == item.max_stock_threshold(),
            || "max mismatch".to_string(),
        )?;
        check(restored.on_reorder() == item.on_reorder(), || {
            "on_reorder mismatch".to_string()
        })
    }

    #[test]
    fn row_to_item_rejects_negative_stock() -> Result<(), Error> {
        let row = CatalogItemRow {
            id: 1,
            name: "x".to_string(),
            description: None,
            price: Decimal::from(1),
            picture_file_name: None,
            brand_id: Uuid::new_v4(),
            kind_id: Uuid::new_v4(),
            available_stock: -1,
            restock_threshold: 0,
            max_stock_threshold: 10,
            on_reorder: false,
        };
        let outcome = row_to_item(&row);
        check(matches!(outcome, Err(Error::NumericRange { .. })), || {
            format!("expected NumericRange, got {outcome:?}")
        })
    }

    #[test]
    fn row_to_item_rejects_empty_name() -> Result<(), Error> {
        let row = CatalogItemRow {
            id: 1,
            name: String::new(),
            description: None,
            price: Decimal::from(1),
            picture_file_name: None,
            brand_id: Uuid::new_v4(),
            kind_id: Uuid::new_v4(),
            available_stock: 1,
            restock_threshold: 0,
            max_stock_threshold: 10,
            on_reorder: false,
        };
        let outcome = row_to_item(&row);
        check(
            matches!(outcome, Err(Error::EmptyString { field: "item name" })),
            || format!("expected EmptyString, got {outcome:?}"),
        )
    }

    #[test]
    fn brand_round_trip() -> Result<(), Error> {
        let brand = CatalogBrand::new(CatalogBrandId::new(), BrandName::try_from(".NET")?);
        let helper = brand_to_row(&brand);
        let row = CatalogBrandRow {
            id: helper.id,
            name: helper.name,
        };
        let restored = row_to_brand(&row)?;
        check(restored.id() == brand.id(), || "id mismatch".to_string())?;
        check(restored.name() == brand.name(), || {
            "name mismatch".to_string()
        })
    }

    #[test]
    fn kind_round_trip() -> Result<(), Error> {
        let kind = CatalogKind::new(CatalogKindId::new(), KindName::try_from("Mug")?);
        let helper = kind_to_row(&kind);
        let row = CatalogKindRow {
            id: helper.id,
            name: helper.name,
        };
        let restored = row_to_kind(&row)?;
        check(restored.id() == kind.id(), || "id mismatch".to_string())?;
        check(restored.name() == kind.name(), || {
            "name mismatch".to_string()
        })
    }
}
