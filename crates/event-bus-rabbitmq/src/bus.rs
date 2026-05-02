//! [`RabbitMqEventBus`] implementation.

use std::marker::PhantomData;
use std::sync::Arc;

use comp_cat_rs::effect::io::Io;
use event_bus::{Error, EventBus, IntegrationEvent};
use lapin::options::{BasicPublishOptions, ExchangeDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Connection, ConnectionProperties, ExchangeKind};
use tokio::runtime::Runtime;

use crate::config::RmqConfig;

/// Persistent delivery mode (per AMQP spec).
const PERSISTENT_DELIVERY_MODE: u8 = 2;

/// RabbitMQ-backed [`EventBus`] implementation.
///
/// Owns a dedicated [`tokio::runtime::Runtime`] and a [`lapin::Connection`].
/// All [`EventBus::publish`] calls drive the broker through the owned
/// runtime.  See module-level docs for thread safety constraints.
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
