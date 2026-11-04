# llm-gateway

An enterprise-grade LLM gateway built with Rust.

## Architecture

The project follows Clean Architecture using a Rust workspace.

### Workspace crates

| Crate | Responsibility |
|---|---|
| gateway-core | Domain model and business rules |
| gateway-application | Use cases and application contracts |
| gateway-infrastructure | Database, Redis, provider, and external adapters |
| gateway-api | HTTP interface and application composition |

## Technology Stack

- Rust
- Axum
- Tokio
- PostgreSQL
- SQLx
- Redis
- OpenTelemetry
- Docker

## Development

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
