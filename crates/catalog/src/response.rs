//! Response DTOs serialized to JSON response bodies.
//!
//! Fields are private; serde derives expand in the same module and
//! have access to them.  External callers construct via [`From`]
//! projections or the explicit constructors below.

use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::brand::CatalogBrand;
use crate::item::CatalogItem;
use crate::kind::CatalogKind;

/// `GET /api/catalog/items/{id}` and the elements of
/// `GET /api/catalog/items` response bodies.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogItemResponse {
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
    on_reorder: bool,
}

impl From<&CatalogItem> for CatalogItemResponse {
    fn from(item: &CatalogItem) -> Self {
        Self {
            id: item.id().into_i32(),
            name: item.name().as_str().to_string(),
            description: item.description().map(|d| d.as_str().to_string()),
            price: item.price().into_decimal(),
            picture_file_name: item.picture_file_name().map(|p| p.as_str().to_string()),
            brand_id: item.brand_id().into_uuid(),
            kind_id: item.kind_id().into_uuid(),
            available_stock: item.available_stock().get(),
            restock_threshold: item.restock_threshold().get(),
            max_stock_threshold: item.max_stock_threshold().get(),
            on_reorder: item.on_reorder(),
        }
    }
}

/// `GET /api/catalog/brands` response element.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogBrandResponse {
    id: Uuid,
    name: String,
}

impl From<&CatalogBrand> for CatalogBrandResponse {
    fn from(brand: &CatalogBrand) -> Self {
        Self {
            id: brand.id().into_uuid(),
            name: brand.name().as_str().to_string(),
        }
    }
}

/// `GET /api/catalog/kinds` response element.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogKindResponse {
    id: Uuid,
    name: String,
}

impl From<&CatalogKind> for CatalogKindResponse {
    fn from(kind: &CatalogKind) -> Self {
        Self {
            id: kind.id().into_uuid(),
            name: kind.name().as_str().to_string(),
        }
    }
}

/// Generic paginated wrapper, mirroring upstream's `PaginatedItems<T>`.
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedItemsResponse<T>
where
    T: Serialize,
{
    page_index: u32,
    page_size: u32,
    count: u64,
    data: Vec<T>,
}

impl<T> PaginatedItemsResponse<T>
where
    T: Serialize,
{
    /// Construct.
    #[must_use]
    pub fn new(page_index: u32, page_size: u32, count: u64, data: Vec<T>) -> Self {
        Self {
            page_index,
            page_size,
            count,
            data,
        }
    }
}

/// `POST /api/catalog/items` / `/brands` / `/kinds` response body.
///
/// Generic over the id width: items are `CreatedResponse<i32>`,
/// brands and kinds are `CreatedResponse<Uuid>`.
#[derive(Debug, Clone, Serialize)]
pub struct CreatedResponse<T: Serialize> {
    id: T,
}

impl<T: Serialize> CreatedResponse<T> {
    /// Wrap a fresh aggregate id for the JSON response.
    #[must_use]
    pub fn new(id: T) -> Self {
        Self { id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    use crate::brand::CatalogBrandId;
    use crate::error::Error;
    use crate::item::CatalogItemId;
    use crate::kind::CatalogKindId;
    use crate::money::Price;
    use crate::stock::{MaxStockThreshold, RestockThreshold, Stock};
    use crate::strings::{BrandName, ItemDescription, ItemName, KindName, PictureFileName};

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
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
    fn item_response_projects_all_fields() -> Result<(), Error> {
        let item = sample_item()?;
        let response = CatalogItemResponse::from(&item);
        check(response.id == item.id().into_i32(), || {
            "id mismatch".to_string()
        })?;
        check(response.name == ".NET Bot Black Hoodie", || {
            format!("name {}", response.name)
        })?;
        check(response.available_stock == 100, || {
            format!("stock {}", response.available_stock)
        })?;
        check(!response.on_reorder, || "on_reorder mismatch".to_string())
    }

    #[test]
    fn brand_response_projects() -> Result<(), Error> {
        let brand = CatalogBrand::new(CatalogBrandId::new(), BrandName::try_from(".NET")?);
        let response = CatalogBrandResponse::from(&brand);
        check(response.name == ".NET", || {
            format!("name {}", response.name)
        })
    }

    #[test]
    fn kind_response_projects() -> Result<(), Error> {
        let kind = CatalogKind::new(CatalogKindId::new(), KindName::try_from("Mug")?);
        let response = CatalogKindResponse::from(&kind);
        check(response.name == "Mug", || format!("name {}", response.name))
    }

    #[test]
    fn paginated_response_serializes() -> Result<(), Error> {
        let item = sample_item()?;
        let response =
            PaginatedItemsResponse::new(0, 10, 1, vec![CatalogItemResponse::from(&item)]);
        let body = serde_json::to_string(&response)?;
        check(body.contains("\"page_index\":0"), || format!("body {body}"))?;
        check(body.contains("\"count\":1"), || format!("body {body}"))
    }

    #[test]
    fn created_response_wraps_id() -> Result<(), Error> {
        let id = Uuid::new_v4();
        let response = CreatedResponse::new(id);
        let body = serde_json::to_string(&response)?;
        let needle = id.to_string();
        check(body.contains(&needle), || format!("body {body}"))
    }
}
