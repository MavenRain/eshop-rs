//! Repository for the [`Buyer`](ordering_domain::Buyer) aggregate.

use ordering_domain::Buyer;

use crate::error::Error;
use crate::mapper::{buyer_to_rows, rows_to_buyer};
use crate::row::{BuyerRow, PaymentMethodRow};

/// Persistence boundary for the [`Buyer`] aggregate.
pub struct BuyerRepository;

impl BuyerRepository {
    /// Insert a new buyer plus its payment methods.
    ///
    /// # Errors
    /// Propagates toasty and mapping errors.
    pub async fn add<E>(executor: &mut E, buyer: &Buyer) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let (head, payments) = buyer_to_rows(buyer)?;
        toasty::create!(BuyerRow {
            id: head.id,
            identity_guid: head.identity_guid,
            name: head.name,
        })
        .exec(executor)
        .await?;
        // Sequential async insert; localized FFI exception (see
        // `event-bus-rabbitmq::bind_all` and `OrderRepository::add`).
        for pm in payments {
            toasty::create!(PaymentMethodRow {
                id: pm.id,
                buyer_id: pm.buyer_id,
                alias: pm.alias,
                card_number: pm.card_number,
                security_number: pm.security_number,
                card_holder_name: pm.card_holder_name,
                expiration: pm.expiration,
                card_type_id: pm.card_type_id,
            })
            .exec(executor)
            .await?;
        }
        Ok(())
    }

    /// Load a buyer by their external identity GUID, including all payment
    /// methods.
    ///
    /// # Errors
    /// Propagates toasty and mapping errors.
    pub async fn find_by_identity_guid<E>(
        executor: &mut E,
        identity_guid: &str,
    ) -> Result<Buyer, Error>
    where
        E: toasty::Executor,
    {
        let row = BuyerRow::get_by_identity_guid(executor, identity_guid).await?;
        let payments: Vec<PaymentMethodRow> = row.payment_methods().exec(executor).await?;
        rows_to_buyer(&row, &payments)
    }
}
