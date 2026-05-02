//! [`RabbitMqEventBus`] implementation.

use std::marker::PhantomData;
use std::sync::Arc;

use comp_cat_rs::effect::io::Io;
use event_bus::{Error, EventBus, EventBusSubscriber, IntegrationEvent};
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Connection, ConnectionProperties, ExchangeKind};
use tokio::runtime::Runtime;

use crate::config::RmqConfig;

/// Persistent delivery mode (per AMQP spec).
const PERSISTENT_DELIVERY_MODE: u8 = 2;

/// Empty consumer tag; lapin generates a unique one for us.
const AUTO_CONSUMER_TAG: &str = "";

/// `RabbitMQ`-backed [`EventBus`] implementation.
///
/// Owns a dedicated [`tokio::runtime::Runtime`] and a [`lapin::Connection`].
/// All [`EventBus::publish`] and [`EventBusSubscriber::subscribe`] calls
/// drive the broker through the owned runtime.  See module-level docs for
/// thread safety constraints.
pub struct RabbitMqEventBus<E> {
    connection: Arc<Connection>,
    runtime: Arc<Runtime>,
    config: RmqConfig,
    _phantom: PhantomData<fn() -> E>,
}

impl<E> RabbitMqEventBus<E> {
    /// Connect to the broker and construct a bus.
    ///
    /// # Errors
    /// Returns `Err(Error::Connection { reason })` if the runtime fails to
    /// build or the broker connection cannot be opened.
    pub fn connect(config: RmqConfig) -> Result<Self, Error> {
        let runtime = Runtime::new().map_err(|e| Error::Connection {
            reason: format!("failed to build tokio runtime: {e}"),
        })?;
        let connection = runtime
            .block_on(async {
                Connection::connect(config.amqp_uri(), ConnectionProperties::default()).await
            })
            .map_err(|e| Error::Connection {
                reason: e.to_string(),
            })?;
        Ok(Self {
            connection: Arc::new(connection),
            runtime: Arc::new(runtime),
            config,
            _phantom: PhantomData,
        })
    }
}

impl<E: IntegrationEvent> EventBus<E> for RabbitMqEventBus<E> {
    fn publish(&self, event: E) -> Io<Error, ()> {
        let routing_key = event.event_name().to_string();
        let payload_result = serde_json::to_vec(&event);
        let connection = self.connection.clone();
        let runtime = self.runtime.clone();
        let exchange = self.config.exchange().as_str().to_string();
        Io::suspend(move || {
            let payload = payload_result.map_err(|e| Error::Serialization {
                reason: e.to_string(),
            })?;
            runtime.block_on(async move {
                let channel = connection
                    .create_channel()
                    .await
                    .map_err(|e| Error::Connection {
                        reason: e.to_string(),
                    })?;
                channel
                    .exchange_declare(
                        &exchange,
                        ExchangeKind::Direct,
                        ExchangeDeclareOptions::default(),
                        FieldTable::default(),
                    )
                    .await
                    .map_err(|e| Error::Publish {
                        reason: e.to_string(),
                    })?;
                let confirm = channel
                    .basic_publish(
                        &exchange,
                        &routing_key,
                        BasicPublishOptions {
                            mandatory: true,
                            ..Default::default()
                        },
                        &payload,
                        BasicProperties::default().with_delivery_mode(PERSISTENT_DELIVERY_MODE),
                    )
                    .await
                    .map_err(|e| Error::Publish {
                        reason: e.to_string(),
                    })?;
                confirm.await.map_err(|e| Error::Publish {
                    reason: e.to_string(),
                })?;
                Ok(())
            })
        })
    }
}

impl<E: IntegrationEvent> EventBusSubscriber<E> for RabbitMqEventBus<E> {
    fn subscribe<F>(&self, handler: F) -> Io<Error, ()>
    where
        F: Fn(E) -> Io<Error, ()> + Send + Sync + 'static,
    {
        let connection = self.connection.clone();
        let runtime = self.runtime.clone();
        let exchange = self.config.exchange().as_str().to_string();
        let queue_name = self.config.queue_name().as_str().to_string();
        let event_names: Vec<&'static str> = E::all_event_names().to_vec();
        Io::suspend(move || {
            let runtime_handle = runtime.clone();
            runtime_handle.block_on(async move {
                let channel = connection
                    .create_channel()
                    .await
                    .map_err(|e| Error::Connection {
                        reason: e.to_string(),
                    })?;
                channel
                    .exchange_declare(
                        &exchange,
                        ExchangeKind::Direct,
                        ExchangeDeclareOptions::default(),
                        FieldTable::default(),
                    )
                    .await
                    .map_err(|e| Error::Subscribe {
                        reason: e.to_string(),
                    })?;
                channel
                    .queue_declare(
                        &queue_name,
                        QueueDeclareOptions {
                            durable: true,
                            ..Default::default()
                        },
                        FieldTable::default(),
                    )
                    .await
                    .map_err(|e| Error::Subscribe {
                        reason: e.to_string(),
                    })?;
                bind_all(&channel, &queue_name, &exchange, &event_names).await?;
                let consumer = channel
                    .basic_consume(
                        &queue_name,
                        AUTO_CONSUMER_TAG,
                        BasicConsumeOptions::default(),
                        FieldTable::default(),
                    )
                    .await
                    .map_err(|e| Error::Subscribe {
                        reason: e.to_string(),
                    })?;
                runtime.spawn(consume_loop::<E, F>(consumer, handler));
                Ok(())
            })
        })
    }
}

/// Bind `queue` to `exchange` for each routing key in `names`.  A `for` loop
/// is used here because async-recursion would require `Box<dyn Future>`,
/// which violates the no-`dyn` rule, and the alternatives (`futures_util::TryStreamExt::try_fold`)
/// add a heavier dep without simplifying the body.  This is a localized
/// FFI-driven exception scoped to one call site.
async fn bind_all(
    channel: &lapin::Channel,
    queue: &str,
    exchange: &str,
    names: &[&'static str],
) -> Result<(), Error> {
    #[allow(clippy::needless_range_loop)]
    for name in names {
        channel
            .queue_bind(
                queue,
                exchange,
                name,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| Error::Subscribe {
                reason: e.to_string(),
            })?;
    }
    Ok(())
}

/// Drive the consumer until the broker closes the stream.  Handler errors,
/// deserialization failures, and ack failures are logged to stderr; the loop
/// continues so a single bad message does not stop the consumer.
async fn consume_loop<E, F>(mut consumer: lapin::Consumer, handler: F)
where
    E: IntegrationEvent,
    F: Fn(E) -> Io<Error, ()> + Send + Sync + 'static,
{
    use futures_lite::stream::StreamExt;
    // `lapin::Consumer` exposes `next` via `StreamExt` requiring `&mut self`,
    // forcing a `mut` binding (FFI-mut exception per the user's no-mut rule).
    while let Some(item) = consumer.next().await {
        process_one::<E, F>(item, &handler).await;
    }
}

/// Handle one delivery: deserialize, dispatch, ack.  Each failure mode logs
/// to stderr; a real deployment should swap this for tracing + a DLX.
async fn process_one<E, F>(item: Result<lapin::message::Delivery, lapin::Error>, handler: &F)
where
    E: IntegrationEvent,
    F: Fn(E) -> Io<Error, ()>,
{
    match item {
        Ok(delivery) => {
            handle_delivery::<E, F>(&delivery, handler);
            ack(&delivery).await;
        }
        Err(err) => eprintln!("event-bus-rabbitmq consumer error: {err}"),
    }
}

fn handle_delivery<E, F>(delivery: &lapin::message::Delivery, handler: &F)
where
    E: IntegrationEvent,
    F: Fn(E) -> Io<Error, ()>,
{
    let outcome: Result<(), Error> = serde_json::from_slice::<E>(&delivery.data)
        .map_err(|err| Error::Deserialization {
            reason: err.to_string(),
        })
        .and_then(|event| handler(event).run());
    let _ = outcome.inspect_err(|err| eprintln!("event-bus-rabbitmq message error: {err}"));
}

async fn ack(delivery: &lapin::message::Delivery) {
    let _ = delivery
        .ack(BasicAckOptions::default())
        .await
        .inspect_err(|err| eprintln!("event-bus-rabbitmq ack error: {err}"));
}
