//! Repository for the [`Order`](ordering_domain::Order) aggregate.

use ordering_domain::{Order, OrderId, OrderStatus};

use crate::error::Error;
use crate::mapper::{order_to_rows, rows_to_order};
use crate::row::{OrderItemRow, OrderRow};

/// Persistence boundary for the [`Order`] aggregate.
pub struct OrderRepository;

impl OrderRepository {
    /// Insert a new order plus its line items.  Caller is responsible for
    /// running this inside a [`toasty::Transaction`] alongside any outbox
    /// writes.
    ///
    /// # Errors
    /// Propagates toasty errors and mapping errors.
    pub async fn add<E>(executor: &mut E, order: &Order) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let (head, items) = order_to_rows(order)?;
        toasty::create!(OrderRow {
            id: head.id,
            order_date: head.order_date,
            order_status: head.order_status,
            description: head.description,
            buyer_id: head.buyer_id,
            payment_method_id: head.payment_method_id,
            street: head.street,
            city: head.city,
            state: head.state,
            country: head.country,
            zip_code: head.zip_code,
        })
        .exec(executor)
        .await?;
        // Sequential async insert.  A `for` is used here for the same
        // FFI-driven reason as `event-bus-rabbitmq::bind_all`: each step
        // needs a fresh `&mut Executor` borrow, which `futures_lite::stream`
        // combinators cannot pass through without `Box<dyn Future>`
        // (no-`dyn` rule) or a heavier dep.
        for item in items {
            toasty::create!(OrderItemRow {
                id: item.id,
                order_id: item.order_id,
                product_id: item.product_id,
                product_name: item.product_name,
                picture_url: item.picture_url,
                unit_price: item.unit_price,
                discount: item.discount,
                units: item.units,
            })
            .exec(executor)
            .await?;
        }
        Ok(())
    }

    /// Load an order by id, including its line items.
    ///
    /// # Errors
    /// Propagates toasty errors and mapping errors.
    pub async fn get_by_id<E>(executor: &mut E, id: OrderId) -> Result<Order, Error>
    where
        E: toasty::Executor,
    {
        let row = OrderRow::get_by_id(executor, &id.into_uuid()).await?;
        let items = row.items().exec(executor).await?;
        let item_vec: Vec<OrderItemRow> = items;
        rows_to_order(&row, &item_vec)
    }

    /// Update only the `order_status` and `description` of an existing order.
    ///
    /// # Errors
    /// Propagates toasty errors.
    pub async fn update_status<E>(
        executor: &mut E,
        id: OrderId,
        status: OrderStatus,
        description: &str,
    ) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        OrderRow::update_by_id(id.into_uuid())
            .order_status(status.to_string())
            .description(description.to_string())
            .exec(executor)
            .await?;
        Ok(())
    }
}
