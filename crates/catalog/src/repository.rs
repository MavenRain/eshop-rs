//! Repositories for the Catalog bounded context.
//!
//! Three persistence boundaries, one per aggregate:
//! [`CatalogItemRepository`], [`CatalogBrandRepository`],
//! [`CatalogKindRepository`].  Each method takes a `&mut E:
//! toasty::Executor`, the FFI exception class documented in
//! `ordering-infrastructure::OrderRepository::add` and
//! `event-bus-rabbitmq::bind_all`.
//!
//! [`CatalogItem`] does not own its [`CatalogBrand`] or [`CatalogKind`]
//! aggregates; foreign-key columns hold the relationship and the API
//! layer is responsible for any join-style fetching.  We deliberately
//! keep [`belongs_to`](toasty::BelongsTo) /
//! [`has_many`](toasty::HasMany) wiring out of the row layer so the
//! aggregate boundary is honoured at the schema level too.

use crate::brand::{CatalogBrand, CatalogBrandId};
use crate::error::Error;
use crate::item::{CatalogItem, CatalogItemId};
use crate::kind::{CatalogKind, CatalogKindId};
use crate::mapper::{
    brand_to_row, item_to_row, kind_to_row, row_to_brand, row_to_item, row_to_kind,
};
use crate::row::{CatalogBrandRow, CatalogItemRow, CatalogKindRow};

/// Persistence boundary for the [`CatalogItem`] aggregate.
pub struct CatalogItemRepository;

impl CatalogItemRepository {
    /// Insert a new catalog item.
    ///
    /// # Errors
    /// Propagates toasty errors and mapping errors.
    pub async fn add<E>(executor: &mut E, item: &CatalogItem) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let row = item_to_row(item);
        toasty::create!(CatalogItemRow {
            id: row.id,
            name: row.name,
            description: row.description,
            price: row.price,
            picture_file_name: row.picture_file_name,
            brand_id: row.brand_id,
            kind_id: row.kind_id,
            available_stock: row.available_stock,
            restock_threshold: row.restock_threshold,
            max_stock_threshold: row.max_stock_threshold,
            on_reorder: row.on_reorder,
        })
        .exec(executor)
        .await?;
        Ok(())
    }

    /// Load an item by id.
    ///
    /// # Errors
    /// Propagates toasty errors (including not-found) and mapping errors.
    pub async fn get_by_id<E>(executor: &mut E, id: CatalogItemId) -> Result<CatalogItem, Error>
    where
        E: toasty::Executor,
    {
        let row = CatalogItemRow::get_by_id(executor, &id.into_i32()).await?;
        row_to_item(&row)
    }

    /// Replace every column of an existing item.
    ///
    /// # Errors
    /// Propagates toasty errors (including not-found) and mapping errors.
    pub async fn update<E>(executor: &mut E, item: &CatalogItem) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let row = item_to_row(item);
        CatalogItemRow::update_by_id(row.id)
            .name(row.name)
            .description(row.description)
            .price(row.price)
            .picture_file_name(row.picture_file_name)
            .brand_id(row.brand_id)
            .kind_id(row.kind_id)
            .available_stock(row.available_stock)
            .restock_threshold(row.restock_threshold)
            .max_stock_threshold(row.max_stock_threshold)
            .on_reorder(row.on_reorder)
            .exec(executor)
            .await?;
        Ok(())
    }

    /// Delete an item by id.  The current toasty surface requires
    /// loading the row before deleting it; that is two queries, but
    /// avoids an `Error::NotFound`-style code path before delete-by-key
    /// is added upstream.
    ///
    /// # Errors
    /// Propagates toasty errors (including not-found).
    pub async fn delete_by_id<E>(executor: &mut E, id: CatalogItemId) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let row = CatalogItemRow::get_by_id(executor, &id.into_i32()).await?;
        row.delete().exec(executor).await?;
        Ok(())
    }

    /// Load every item.  Pagination is handled at the API layer.
    ///
    /// # Errors
    /// Propagates toasty errors and mapping errors.
    pub async fn list_all<E>(executor: &mut E) -> Result<Vec<CatalogItem>, Error>
    where
        E: toasty::Executor,
    {
        let rows: Vec<CatalogItemRow> = CatalogItemRow::all().exec(executor).await?;
        rows.iter().map(row_to_item).collect()
    }
}

/// Persistence boundary for the [`CatalogBrand`] aggregate.
pub struct CatalogBrandRepository;

impl CatalogBrandRepository {
    /// Insert a new brand.
    ///
    /// # Errors
    /// Propagates toasty errors.
    pub async fn add<E>(executor: &mut E, brand: &CatalogBrand) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let row = brand_to_row(brand);
        toasty::create!(CatalogBrandRow {
            id: row.id,
            name: row.name,
        })
        .exec(executor)
        .await?;
        Ok(())
    }

    /// Load a brand by id.
    ///
    /// # Errors
    /// Propagates toasty errors (including not-found) and mapping errors.
    pub async fn get_by_id<E>(executor: &mut E, id: CatalogBrandId) -> Result<CatalogBrand, Error>
    where
        E: toasty::Executor,
    {
        let row = CatalogBrandRow::get_by_id(executor, &id.into_uuid()).await?;
        row_to_brand(&row)
    }

    /// Load every brand.
    ///
    /// # Errors
    /// Propagates toasty errors and mapping errors.
    pub async fn list_all<E>(executor: &mut E) -> Result<Vec<CatalogBrand>, Error>
    where
        E: toasty::Executor,
    {
        let rows: Vec<CatalogBrandRow> = CatalogBrandRow::all().exec(executor).await?;
        rows.iter().map(row_to_brand).collect()
    }
}

/// Persistence boundary for the [`CatalogKind`] aggregate.
pub struct CatalogKindRepository;

impl CatalogKindRepository {
    /// Insert a new kind.
    ///
    /// # Errors
    /// Propagates toasty errors.
    pub async fn add<E>(executor: &mut E, kind: &CatalogKind) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let row = kind_to_row(kind);
        toasty::create!(CatalogKindRow {
            id: row.id,
            name: row.name,
        })
        .exec(executor)
        .await?;
        Ok(())
    }

    /// Load a kind by id.
    ///
    /// # Errors
    /// Propagates toasty errors (including not-found) and mapping errors.
    pub async fn get_by_id<E>(executor: &mut E, id: CatalogKindId) -> Result<CatalogKind, Error>
    where
        E: toasty::Executor,
    {
        let row = CatalogKindRow::get_by_id(executor, &id.into_uuid()).await?;
        row_to_kind(&row)
    }

    /// Load every kind.
    ///
    /// # Errors
    /// Propagates toasty errors and mapping errors.
    pub async fn list_all<E>(executor: &mut E) -> Result<Vec<CatalogKind>, Error>
    where
        E: toasty::Executor,
    {
        let rows: Vec<CatalogKindRow> = CatalogKindRow::all().exec(executor).await?;
        rows.iter().map(row_to_kind).collect()
    }
}
