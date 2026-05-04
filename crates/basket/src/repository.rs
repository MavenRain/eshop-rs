//! Persistence boundary for the [`CustomerBasket`] aggregate.
//!
//! Three operations: load, replace, delete.  Replace deletes the
//! existing item rows for a customer and inserts the new ones, with
//! the customer-keyed [`CustomerBasketRow`] upserted around them.
//! All within a single toasty [`Transaction`](toasty::Transaction).
//!
//! Sequential-async toasty inserts and deletes are concentrated in
//! the per-row-type batch helpers at the bottom of this module: each
//! iteration consumes `&mut executor`, so the FFI exception lives
//! once per helper instead of bubbling up to the API layer.

use uuid::Uuid;

use crate::basket::CustomerBasket;
use crate::customer::CustomerId;
use crate::error::Error;
use crate::mapper::{NewBasketItemRow, basket_to_rows, rows_to_basket};
use crate::row::{BasketItemRow, CustomerBasketRow};

/// Persistence boundary for the [`CustomerBasket`] aggregate.
pub struct BasketRepository;

impl BasketRepository {
    /// Load the basket for `customer_id`.  Returns the toasty error
    /// from `get_by_id` (typically a not-found) when no
    /// [`CustomerBasketRow`] exists.
    ///
    /// # Errors
    /// Propagates toasty errors and mapping errors.
    pub async fn get_by_customer_id<E>(
        executor: &mut E,
        customer_id: CustomerId,
    ) -> Result<CustomerBasket, Error>
    where
        E: toasty::Executor,
    {
        let id = customer_id.into_uuid();
        let customer_row = CustomerBasketRow::get_by_id(executor, &id).await?;
        let item_rows: Vec<BasketItemRow> = toasty::query!(
            BasketItemRow FILTER .customer_basket_id == #id
        )
        .exec(executor)
        .await?;
        rows_to_basket(&customer_row, &item_rows)
    }

    /// Replace the basket for `basket.customer_id()`: delete the
    /// existing customer row and items, then insert the new pair.
    ///
    /// # Errors
    /// Propagates toasty errors and mapping errors.
    pub async fn save<E>(executor: &mut E, basket: &CustomerBasket) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let customer_id = basket.customer_id().into_uuid();
        Self::delete_existing(executor, customer_id).await?;
        let (new_customer_row, new_item_rows) = basket_to_rows(basket);
        toasty::create!(CustomerBasketRow {
            id: new_customer_row.id,
        })
        .exec(executor)
        .await?;
        create_basket_item_rows(executor, new_item_rows).await
    }

    /// Delete the basket and all its items.  Idempotent: deleting a
    /// non-existent basket succeeds.
    ///
    /// # Errors
    /// Propagates toasty errors.
    pub async fn delete_by_customer_id<E>(
        executor: &mut E,
        customer_id: CustomerId,
    ) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        Self::delete_existing(executor, customer_id.into_uuid()).await
    }

    async fn delete_existing<E>(executor: &mut E, customer_id: Uuid) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let item_rows: Vec<BasketItemRow> = toasty::query!(
            BasketItemRow FILTER .customer_basket_id == #customer_id
        )
        .exec(executor)
        .await?;
        delete_basket_item_rows(executor, item_rows).await?;
        // Lift the optional row into a 0-or-1 element Vec via
        // `.ok().into_iter().collect()` so the customer-row delete
        // flows through the same batch-helper shape as the items.
        let customer_rows: Vec<CustomerBasketRow> =
            CustomerBasketRow::get_by_id(executor, &customer_id)
                .await
                .ok()
                .into_iter()
                .collect();
        delete_customer_basket_rows(executor, customer_rows).await
    }
}

/// Insert a batch of [`BasketItemRow`]s, FK-linked to the same
/// customer.  Encapsulates the FFI sequential-async-insert
/// exception so [`BasketRepository::save`] stays combinator-shaped.
///
/// # Errors
/// Propagates toasty errors; first error short-circuits the batch.
async fn create_basket_item_rows<E>(
    executor: &mut E,
    rows: Vec<NewBasketItemRow>,
) -> Result<(), Error>
where
    E: toasty::Executor,
{
    for row in rows {
        toasty::create!(BasketItemRow {
            id: row.id,
            customer_basket_id: row.customer_basket_id,
            product_id: row.product_id,
            product_name: row.product_name,
            unit_price: row.unit_price,
            old_unit_price: row.old_unit_price,
            quantity: row.quantity,
            picture_url: row.picture_url,
        })
        .exec(executor)
        .await?;
    }
    Ok(())
}

/// Delete a batch of loaded [`BasketItemRow`]s.  Encapsulates the
/// FFI sequential-async-delete exception.
///
/// # Errors
/// Propagates toasty errors; first error short-circuits the batch.
async fn delete_basket_item_rows<E>(executor: &mut E, rows: Vec<BasketItemRow>) -> Result<(), Error>
where
    E: toasty::Executor,
{
    for row in rows {
        row.delete().exec(executor).await?;
    }
    Ok(())
}

/// Delete a batch of loaded [`CustomerBasketRow`]s.  In practice the
/// vector is 0-or-1 elements (one customer per save/delete), but the
/// shape mirrors [`delete_basket_item_rows`] so call sites compose
/// the two via the same `into_iter().collect()` →`batch_helper`
/// pattern.
///
/// # Errors
/// Propagates toasty errors.
async fn delete_customer_basket_rows<E>(
    executor: &mut E,
    rows: Vec<CustomerBasketRow>,
) -> Result<(), Error>
where
    E: toasty::Executor,
{
    for row in rows {
        row.delete().exec(executor).await?;
    }
    Ok(())
}
