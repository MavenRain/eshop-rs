//! Request DTOs deserialized from JSON bodies and query strings.
//!
//! Fields are private; serde derives expand in the same module and
//! have access to them.  Public projection methods convert each
//! request into the corresponding domain type, validating along the
//! way.

use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::brand::{CatalogBrand, CatalogBrandId};
use crate::error::Error;
use crate::item::{CatalogItem, CatalogItemId};
use crate::kind::{CatalogKind, CatalogKindId};
use crate::money::Price;
use crate::stock::{MaxStockThreshold, RestockThreshold, Stock};
use crate::strings::{BrandName, ItemDescription, ItemName, KindName, PictureFileName};

/// `POST /api/catalog/items` body.
///
/// `id` is caller-supplied and aligned with upstream eShop's
/// `Catalog.API.CatalogItem.Id` width (`int`).  The basket and
/// ordering integration events carry the same primitive, so the
/// caller (or an upstream sequence service) picks an id that those
/// other contexts can resolve.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemRequest {
    id: i32,
    name: String,
    description: Option<String>,
    price: Decimal,
    picture_file_name: Option<String>,
    brand_id: Uuid,
    kind_id: Uuid,
    available_stock: u32,
    restock_threshold: u32,
    max_stock_threshold: u32,
}

impl CreateItemRequest {
    /// Project this request into a [`CatalogItem`] aggregate.
    ///
    /// # Errors
    /// Returns [`Error::EmptyString`] for blank strings,
    /// [`Error::NegativePrice`] for a negative price, or
    /// [`Error::RestockExceedsMax`] / [`Error::InitialStockExceedsMax`]
    /// for stock-invariant breaks.
    pub fn try_into_item(self) -> Result<CatalogItem, Error> {
        let description = self
            .description
            .map(ItemDescription::try_from)
            .transpose()?;
        let picture_file_name = self
            .picture_file_name
            .map(PictureFileName::try_from)
            .transpose()?;
        CatalogItem::new(
            CatalogItemId::from(self.id),
            ItemName::try_from(self.name)?,
            description,
            Price::new(self.price)?,
            picture_file_name,
            CatalogBrandId::from(self.brand_id),
            CatalogKindId::from(self.kind_id),
            Stock::from(self.available_stock),
            RestockThreshold::from(self.restock_threshold),
            MaxStockThreshold::from(self.max_stock_threshold),
        )
    }
}

/// `PUT /api/catalog/items/:id` body.
///
/// All fields required: this is a wholesale replacement of the item's
/// updatable surface.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateItemRequest {
    name: String,
    description: Option<String>,
    price: Decimal,
    picture_file_name: Option<String>,
    brand_id: Uuid,
    kind_id: Uuid,
    available_stock: u32,
    restock_threshold: u32,
    max_stock_threshold: u32,
}

impl UpdateItemRequest {
    /// Apply this request's fields onto an existing
    /// [`CatalogItem`], emitting a `ProductPriceChanged` event iff
    /// the price differs.
    ///
    /// # Errors
    /// Same set as [`CreateItemRequest::try_into_item`].
    pub fn apply_to(self, existing: CatalogItem) -> Result<CatalogItem, Error> {
        let description = self
            .description
            .map(ItemDescription::try_from)
            .transpose()?;
        let picture_file_name = self
            .picture_file_name
            .map(PictureFileName::try_from)
            .transpose()?;
        existing.apply_update(
            ItemName::try_from(self.name)?,
            description,
            Price::new(self.price)?,
            picture_file_name,
            CatalogBrandId::from(self.brand_id),
            CatalogKindId::from(self.kind_id),
            Stock::from(self.available_stock),
            RestockThreshold::from(self.restock_threshold),
            MaxStockThreshold::from(self.max_stock_threshold),
        )
    }
}

/// `POST /api/catalog/brands` body.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBrandRequest {
    name: String,
}

impl CreateBrandRequest {
    /// Project into a fresh [`CatalogBrand`] aggregate with a freshly
    /// generated id.
    ///
    /// # Errors
    /// [`Error::EmptyString`] if `name` is blank.
    pub fn try_into_brand(self) -> Result<CatalogBrand, Error> {
        Ok(CatalogBrand::new(
            CatalogBrandId::new(),
            BrandName::try_from(self.name)?,
        ))
    }
}

/// `POST /api/catalog/kinds` body.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateKindRequest {
    name: String,
}

impl CreateKindRequest {
    /// Project into a fresh [`CatalogKind`] aggregate with a freshly
    /// generated id.
    ///
    /// # Errors
    /// [`Error::EmptyString`] if `name` is blank.
    pub fn try_into_kind(self) -> Result<CatalogKind, Error> {
        Ok(CatalogKind::new(
            CatalogKindId::new(),
            KindName::try_from(self.name)?,
        ))
    }
}

/// `GET /api/catalog/items?page_size=&page_index=` query string.
///
/// Mirrors upstream's `PaginationRequest(int PageSize=10, int PageIndex=0)`.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PaginationQuery {
    page_size: Option<u32>,
    page_index: Option<u32>,
}

impl PaginationQuery {
    /// Resolved page size, defaulting to upstream's value of 10.
    #[must_use]
    pub fn page_size(&self) -> u32 {
        self.page_size.unwrap_or(10)
    }

    /// Resolved page index, defaulting to upstream's value of 0.
    #[must_use]
    pub fn page_index(&self) -> u32 {
        self.page_index.unwrap_or(0)
    }
}

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

    fn valid_create_item() -> CreateItemRequest {
        CreateItemRequest {
            id: 1,
            name: ".NET Bot Black Hoodie".to_string(),
            description: Some("Stylish hoodie".to_string()),
            price: Decimal::new(1999, 2),
            picture_file_name: Some("hoodie.png".to_string()),
            brand_id: Uuid::new_v4(),
            kind_id: Uuid::new_v4(),
            available_stock: 100,
            restock_threshold: 20,
            max_stock_threshold: 200,
        }
    }

    #[test]
    fn create_item_request_projects_to_item() -> Result<(), Error> {
        let item = valid_create_item().try_into_item()?;
        check(item.name().as_str() == ".NET Bot Black Hoodie", || {
            format!("name {}", item.name().as_str())
        })?;
        check(item.available_stock().get() == 100, || {
            format!("stock {}", item.available_stock().get())
        })
    }

    #[test]
    fn create_item_request_rejects_empty_name() -> Result<(), Error> {
        let request = CreateItemRequest {
            name: String::new(),
            ..valid_create_item()
        };
        let outcome = request.try_into_item();
        check(
            matches!(outcome, Err(Error::EmptyString { field: "item name" })),
            || format!("expected EmptyString, got {outcome:?}"),
        )
    }

    #[test]
    fn create_item_request_rejects_negative_price() -> Result<(), Error> {
        let request = CreateItemRequest {
            price: Decimal::NEGATIVE_ONE,
            ..valid_create_item()
        };
        let outcome = request.try_into_item();
        check(matches!(outcome, Err(Error::NegativePrice)), || {
            format!("expected NegativePrice, got {outcome:?}")
        })
    }

    #[test]
    fn create_item_request_rejects_invalid_thresholds() -> Result<(), Error> {
        let request = CreateItemRequest {
            restock_threshold: 500,
            max_stock_threshold: 100,
            available_stock: 0,
            ..valid_create_item()
        };
        let outcome = request.try_into_item();
        check(
            matches!(
                outcome,
                Err(Error::RestockExceedsMax {
                    restock: 500,
                    max: 100
                })
            ),
            || format!("expected RestockExceedsMax, got {outcome:?}"),
        )
    }

    #[test]
    fn pagination_defaults() -> Result<(), Error> {
        let q = PaginationQuery {
            page_size: None,
            page_index: None,
        };
        check(q.page_size() == 10, || format!("size {}", q.page_size()))?;
        check(q.page_index() == 0, || format!("index {}", q.page_index()))
    }

    #[test]
    fn pagination_overrides() -> Result<(), Error> {
        let q = PaginationQuery {
            page_size: Some(25),
            page_index: Some(2),
        };
        check(q.page_size() == 25, || format!("size {}", q.page_size()))?;
        check(q.page_index() == 2, || format!("index {}", q.page_index()))
    }

    #[test]
    fn create_brand_request_projects() -> Result<(), Error> {
        let request = CreateBrandRequest {
            name: ".NET".to_string(),
        };
        let brand = request.try_into_brand()?;
        check(brand.name().as_str() == ".NET", || {
            format!("name {}", brand.name().as_str())
        })
    }

    #[test]
    fn create_kind_request_projects() -> Result<(), Error> {
        let request = CreateKindRequest {
            name: "Mug".to_string(),
        };
        let kind = request.try_into_kind()?;
        check(kind.name().as_str() == "Mug", || {
            format!("name {}", kind.name().as_str())
        })
    }
}
