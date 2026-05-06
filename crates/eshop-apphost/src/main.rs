//! `eShop` `AppHost` binary.
//!
//! Replaces upstream's .NET Aspire `eShop.AppHost`: a single tokio
//! process that owns the toasty [`Db`], constructs the per-context
//! [`RabbitMqEventBus`] handles, and spawns four concurrent tasks:
//!
//! 1. The combined [`axum`] HTTP server, mounting the ordering,
//!    catalog, and basket routers under one [`Router`].
//! 2. The Ordering outbox publisher loop.
//! 3. The Catalog outbox publisher loop.
//! 4. The Basket outbox publisher loop.
//!
//! All three publisher loops are driven combinator-style via
//! [`futures_lite::stream::unfold`]: each cycle calls
//! [`drain_once`](ordering_processor::OrderProcessor::drain_once)
//! (resp. [`catalog_processor::CatalogProcessor::drain_once`],
//! [`basket_processor::BasketProcessor::drain_once`]),
//! logs any persistence error, and sleeps for the configured poll
//! interval.  The [`Stream`](futures_lite::Stream) is then drained by
//! [`for_each`](futures_lite::stream::StreamExt::for_each), which is
//! the one combinator that turns an infinite stream of `()` items into
//! the long-running task body.
//!
//! Configuration is loaded once at startup from environment variables
//! ([`Config::from_env`]); see `config.rs` for variable names and
//! defaults.

mod config;
mod error;

use std::sync::Arc;

use axum::Router;
use basket::row::{BasketIntegrationEventLogRow, BasketItemRow, CustomerBasketRow};
use basket_integration_events::BasketIntegrationEvent;
use basket_processor::BasketProcessor;
use catalog::row::{
    CatalogBrandRow, CatalogIntegrationEventLogRow, CatalogItemRow, CatalogKindRow,
};
use catalog_integration_events::CatalogIntegrationEvent;
use catalog_processor::CatalogProcessor;
use catalog_subscribers::CatalogConsumedOrderingEvent;
use event_bus::{EventBus, EventBusSubscriber};
use event_bus_rabbitmq::RabbitMqEventBus;
use futures_lite::stream::StreamExt;
use identity::row::UserRow;
use ordering_infrastructure::row::{
    BuyerRow, IntegrationEventLogRow, OrderItemRow, OrderRow, PaymentMethodRow,
};
use ordering_integration_events::OrderingIntegrationEvent;
use ordering_processor::OrderProcessor;
use ordering_subscribers::OrderingConsumedBasketEvent;
use toasty::Db;
use toasty::db::Driver;
use toasty_driver_postgresql::PostgreSQL;
use toasty_driver_sqlite::Sqlite;
use webhooks::row::WebhookSubscriptionRow;
use webhooks_delivery::make_handler as make_delivery_handler;

use crate::config::Config;
use crate::error::Error;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Error> {
    let config = Config::from_env()?;
    let db = Arc::new(build_db(config.database_url()).await?);

    let ordering_state = ordering_api::AppState::new(db.clone());
    let catalog_state = catalog::AppState::new(db.clone());
    let basket_state = basket::AppState::new(db.clone());
    let webhooks_state = webhooks::AppState::new(db.clone());
    let identity_state = identity::AppState::new(db.clone(), config.jwt_issuer());

    let ordering_bus =
        RabbitMqEventBus::<OrderingIntegrationEvent>::connect(config.ordering_rmq())?;
    let catalog_bus = RabbitMqEventBus::<CatalogIntegrationEvent>::connect(config.catalog_rmq())?;
    let basket_bus = RabbitMqEventBus::<BasketIntegrationEvent>::connect(config.basket_rmq())?;

    // Catalog-side subscriber to ordering events.  Bound at startup
    // and held for the lifetime of `main`: the bus owns the runtime
    // that drives the consume loop, so dropping it would cancel the
    // subscription.  The handler closure captures `db` via
    // `make_handler` so it can resolve and decrement catalog items
    // from inside the bus's runtime.
    let catalog_subscriber_bus =
        RabbitMqEventBus::<CatalogConsumedOrderingEvent>::connect(config.catalog_subscriber_rmq())?;
    catalog_subscriber_bus
        .subscribe(catalog_subscribers::make_handler(db.clone()))
        .run()?;

    // Ordering-side subscriber to basket events.  Same lifecycle
    // contract as `catalog_subscriber_bus`; the handler captures
    // `db` via `make_handler` so it can mint and persist orders
    // from inside the bus's runtime.
    let ordering_subscriber_bus =
        RabbitMqEventBus::<OrderingConsumedBasketEvent>::connect(config.ordering_subscriber_rmq())?;
    ordering_subscriber_bus
        .subscribe(ordering_subscribers::make_handler(db.clone()))
        .run()?;

    // Webhooks delivery: one bus per integration-event family,
    // bound to its own queue under the `webhooks-delivery` prefix.
    // All three closures share the same shared HTTP client.
    let http_client = Arc::new(reqwest::Client::new());
    let webhooks_delivery_ordering_bus = RabbitMqEventBus::<OrderingIntegrationEvent>::connect(
        config.webhooks_delivery_rmq("ordering"),
    )?;
    webhooks_delivery_ordering_bus
        .subscribe(make_delivery_handler::<OrderingIntegrationEvent>(
            db.clone(),
            http_client.clone(),
        ))
        .run()?;
    let webhooks_delivery_catalog_bus = RabbitMqEventBus::<CatalogIntegrationEvent>::connect(
        config.webhooks_delivery_rmq("catalog"),
    )?;
    webhooks_delivery_catalog_bus
        .subscribe(make_delivery_handler::<CatalogIntegrationEvent>(
            db.clone(),
            http_client.clone(),
        ))
        .run()?;
    let webhooks_delivery_basket_bus = RabbitMqEventBus::<BasketIntegrationEvent>::connect(
        config.webhooks_delivery_rmq("basket"),
    )?;
    webhooks_delivery_basket_bus
        .subscribe(make_delivery_handler::<BasketIntegrationEvent>(
            db.clone(),
            http_client.clone(),
        ))
        .run()?;

    let ordering_processor = OrderProcessor::new(ordering_bus, db.clone(), config.poll_interval());
    let catalog_processor = CatalogProcessor::new(catalog_bus, db.clone(), config.poll_interval());
    let basket_processor = BasketProcessor::new(basket_bus, db.clone(), config.poll_interval());

    let app = build_router(
        ordering_state,
        catalog_state,
        basket_state,
        webhooks_state,
        identity_state,
    );
    let bind_address = config.bind_address().to_string();

    let http = tokio::spawn(serve_http(app, bind_address));
    let ordering_loop = tokio::spawn(run_ordering_loop(ordering_processor));
    let catalog_loop = tokio::spawn(run_catalog_loop(catalog_processor));
    let basket_loop = tokio::spawn(run_basket_loop(basket_processor));

    let (http_outcome, ordering_outcome, catalog_outcome, basket_outcome) =
        tokio::join!(http, ordering_loop, catalog_loop, basket_loop);
    http_outcome.map_err(|e| Error::Join {
        reason: format!("http: {e}"),
    })??;
    ordering_outcome.map_err(|e| Error::Join {
        reason: format!("ordering: {e}"),
    })?;
    catalog_outcome.map_err(|e| Error::Join {
        reason: format!("catalog: {e}"),
    })?;
    basket_outcome.map_err(|e| Error::Join {
        reason: format!("basket: {e}"),
    })?;
    Ok(())
}

/// Build a toasty [`Db`] with every row model from each bounded
/// context registered.  Driver selection comes from `database_url`:
///
/// | URL prefix                            | Driver                  |
/// |---------------------------------------|-------------------------|
/// | `sqlite:` (incl. `sqlite::memory:`)   | `toasty-driver-sqlite`  |
/// | `postgresql:` / `postgres:`           | `toasty-driver-postgresql` |
///
/// Both branches register the identical model set; only the driver
/// type differs.
async fn build_db(database_url: &str) -> Result<Db, Error> {
    let scheme = scheme_of(database_url);
    match scheme {
        DatabaseScheme::Sqlite => {
            let driver = Sqlite::new(database_url).map_err(|e| Error::Toasty {
                reason: format!("sqlite driver init: {e}"),
            })?;
            build_with_driver(driver).await
        }
        DatabaseScheme::Postgres => {
            let driver = PostgreSQL::new(database_url).map_err(|e| Error::Toasty {
                reason: format!("postgresql driver init: {e}"),
            })?;
            build_with_driver(driver).await
        }
        DatabaseScheme::Unsupported(prefix) => Err(Error::Config {
            reason: format!(
                "ESHOP_DATABASE_URL has unsupported scheme {prefix:?}; \
                 expected `sqlite:`, `postgresql:`, or `postgres:`"
            ),
        }),
    }
}

async fn build_with_driver<D: Driver>(driver: D) -> Result<Db, Error> {
    let db = Db::builder()
        .models(toasty::models!(
            OrderRow,
            OrderItemRow,
            BuyerRow,
            PaymentMethodRow,
            IntegrationEventLogRow,
            CatalogItemRow,
            CatalogBrandRow,
            CatalogKindRow,
            CatalogIntegrationEventLogRow,
            CustomerBasketRow,
            BasketItemRow,
            BasketIntegrationEventLogRow,
            WebhookSubscriptionRow,
            UserRow,
        ))
        .build(driver)
        .await?;
    Ok(db)
}

/// Recognized database URL schemes.
#[derive(Debug, PartialEq, Eq)]
enum DatabaseScheme {
    Sqlite,
    Postgres,
    Unsupported(String),
}

fn scheme_of(url: &str) -> DatabaseScheme {
    let prefix = url.split(':').next().unwrap_or_default();
    match prefix {
        "sqlite" => DatabaseScheme::Sqlite,
        "postgresql" | "postgres" => DatabaseScheme::Postgres,
        other => DatabaseScheme::Unsupported(other.to_string()),
    }
}

/// Build the combined axum router that hosts every API route.
///
/// All five routers are mounted at the root; the crates pick
/// non-overlapping prefixes (`/orders*` for ordering, `/api/catalog/*`
/// for catalog, `/api/basket/*` for basket, `/api/webhooks*` for
/// webhooks, `/api/identity/*` for identity), so successive
/// [`Router::merge`] calls are enough.
fn build_router(
    ordering_state: ordering_api::AppState,
    catalog_state: catalog::AppState,
    basket_state: basket::AppState,
    webhooks_state: webhooks::AppState,
    identity_state: identity::AppState,
) -> Router {
    Router::new()
        .merge(ordering_api::build_router(ordering_state))
        .merge(catalog::build_router(catalog_state))
        .merge(basket::build_router(basket_state))
        .merge(webhooks::build_router(webhooks_state))
        .merge(identity::build_router(identity_state))
}

/// Bind the listener and run the axum server until the process is
/// signalled.
async fn serve_http(app: Router, bind_address: String) -> Result<(), Error> {
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .map_err(|e| Error::Bind {
            reason: format!("{bind_address}: {e}"),
        })?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| Error::Serve {
            reason: e.to_string(),
        })
}

/// Resolve when the process receives SIGINT (Ctrl+C).
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .ok()
        .iter()
        .for_each(|()| eprintln!("ctrl-c received, shutting down"));
}

/// Drive [`OrderProcessor::drain_once`] on the configured cadence
/// forever.  Persistence errors are logged to stderr; the loop never
/// exits on its own (the surrounding tokio task is cancelled at
/// shutdown).
async fn run_ordering_loop<B>(processor: OrderProcessor<B>)
where
    B: EventBus<OrderingIntegrationEvent> + Send + Sync + 'static,
{
    futures_lite::stream::unfold(processor, |proc| async move {
        let outcome = proc.drain_once().await;
        outcome
            .as_ref()
            .err()
            .iter()
            .for_each(|err| eprintln!("ordering drain failed: {err}"));
        tokio::time::sleep(proc.poll_interval()).await;
        Some(((), proc))
    })
    .for_each(|()| ())
    .await;
}

/// Drive [`CatalogProcessor::drain_once`] on the configured cadence
/// forever.  Mirror of [`run_ordering_loop`].
async fn run_catalog_loop<B>(processor: CatalogProcessor<B>)
where
    B: EventBus<CatalogIntegrationEvent> + Send + Sync + 'static,
{
    futures_lite::stream::unfold(processor, |proc| async move {
        let outcome = proc.drain_once().await;
        outcome
            .as_ref()
            .err()
            .iter()
            .for_each(|err| eprintln!("catalog drain failed: {err}"));
        tokio::time::sleep(proc.poll_interval()).await;
        Some(((), proc))
    })
    .for_each(|()| ())
    .await;
}

/// Drive [`BasketProcessor::drain_once`] on the configured cadence
/// forever.  Mirror of [`run_ordering_loop`].
async fn run_basket_loop<B>(processor: BasketProcessor<B>)
where
    B: EventBus<BasketIntegrationEvent> + Send + Sync + 'static,
{
    futures_lite::stream::unfold(processor, |proc| async move {
        let outcome = proc.drain_once().await;
        outcome
            .as_ref()
            .err()
            .iter()
            .for_each(|err| eprintln!("basket drain failed: {err}"));
        tokio::time::sleep(proc.poll_interval()).await;
        Some(((), proc))
    })
    .for_each(|()| ())
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Config { reason: reason() })
        }
    }

    #[test]
    fn scheme_of_recognizes_sqlite_in_memory() -> Result<(), Error> {
        let scheme = scheme_of("sqlite::memory:");
        check(scheme == DatabaseScheme::Sqlite, || {
            format!("got {scheme:?}")
        })
    }

    #[test]
    fn scheme_of_recognizes_sqlite_file() -> Result<(), Error> {
        let scheme = scheme_of("sqlite:/tmp/eshop.db");
        check(scheme == DatabaseScheme::Sqlite, || {
            format!("got {scheme:?}")
        })
    }

    #[test]
    fn scheme_of_recognizes_postgresql() -> Result<(), Error> {
        let scheme = scheme_of("postgresql://eshop:eshop@localhost/eshop");
        check(scheme == DatabaseScheme::Postgres, || {
            format!("got {scheme:?}")
        })
    }

    #[test]
    fn scheme_of_recognizes_postgres_short_form() -> Result<(), Error> {
        let scheme = scheme_of("postgres://eshop:eshop@localhost/eshop");
        check(scheme == DatabaseScheme::Postgres, || {
            format!("got {scheme:?}")
        })
    }

    #[test]
    fn scheme_of_rejects_unknown_scheme() -> Result<(), Error> {
        let scheme = scheme_of("mysql://localhost/eshop");
        check(
            matches!(&scheme, DatabaseScheme::Unsupported(s) if s == "mysql"),
            || format!("got {scheme:?}"),
        )
    }

    #[test]
    fn scheme_of_rejects_url_with_no_scheme() -> Result<(), Error> {
        let scheme = scheme_of("localhost/eshop");
        check(matches!(scheme, DatabaseScheme::Unsupported(_)), || {
            format!("got {scheme:?}")
        })
    }
}
