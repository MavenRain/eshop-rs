# eshop-rs

Rust port of [`dotnet/eShop`](https://github.com/dotnet/eShop), the Microsoft .NET reference retail microservices application.

## Status

In progress.  Workstream 1 of the CFA Visibility Portfolio.

## Foundation

- [`comp-cat-rs`](https://github.com/MavenRain/comp-cat-rs) for effects (`Io`, `Stream`, `Resource`, `Fiber`).
- [`toasty`](https://github.com/tokio-rs/toasty) for persistence (PostgreSQL).
- [`lapin`](https://crates.io/crates/lapin) for the event bus (RabbitMQ parity with upstream eShop).
- Hand-rolled Rust orchestrator replaces .NET Aspire.

## Architecture

The port mirrors eShop's bounded contexts as a Cargo workspace.  Crates land in punchlist order:

| Crate | Upstream | Status |
|---|---|---|
| `ordering-domain` | `Ordering.Domain` | scaffolded |
| `event-bus` + `event-bus-rabbitmq` | `EventBus` + `EventBusRabbitMQ` | scaffolded |
| `ordering-infrastructure` | `Ordering.Infrastructure` + `IntegrationEventLogEF` | scaffolded |
| `ordering-api` | `Ordering.API` | scaffolded |
| `catalog` | `Catalog.API` | pending |
| `basket` | `Basket.API` | pending |
| `identity` | `Identity.API` | pending |
| `order-processor` | `OrderProcessor` | pending |
| `payment-processor` | `PaymentProcessor` | pending |
| `webhooks` + `webhook-client` | `Webhooks.API` + `WebhookClient` | pending |
| `app-host` | `eShop.AppHost` | pending |

The web frontend lives in `web-app-elm/`, an [Elm](https://elm-lang.org/) SPA replacing the Blazor `WebApp` project from upstream eShop.

## License

Dual-licensed under either of

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
