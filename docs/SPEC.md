# SPEC.md - BioViva Executable Specification

## 1. Executive Target & System Boundary

- **Project**: BioViva – e-commerce platform for premium organic, bio-certified products
- **Objective**: High-performance, secure platform with full batch traceability (origin, harvest date, acidity/purity metrics) and seamless digital purchasing
- **Tech Stack**: Rust (Axum, Tokio, SQLx), PostgreSQL (single instance, schemas: `auth`, `catalog`, `sales`, `inventory`), Docker
- **Architecture**: Domain-Driven Design (DDD) & Clean Architecture across 4 Cargo workspace crates
- **Entry Point**: `crates/api/src/main.rs`
- **Currency**: Fixed EUR for MVP (no multi-currency conversion)
- **Out of Scope (Phase 1)**: Multi-vendor payouts, AI recommendations, international multi-currency

## 2. Active Invariants & Quality Gates

| Invariant | Rule | Enforcement |
|---|---|---|
| **Domain Purity** | `crates/domain` has zero external IO dependencies | `cargo check --package bioivara-domain` must compile without `sqlx`, `axum`, `tokio` features |
| **SQL Safety** | All database queries use SQLx parameterized queries only | No raw string concatenation for SQL values; `sqlx::query!` / `query_as!` with `$1`, `$2` placeholders |
| **Auth Security** | Passwords Argon2-hashed; JWT signed with HMAC-SHA256 | Argon2id memory=64MiB, time=3, parallelism=2; JWT secrets rotated per environment |
| **Schema Isolation** | Tables physically separated into `auth`, `catalog`, `sales`, `inventory` schemas | Cross-schema joins only via documented read models; `SET search_path` per connection |
| **Currency Fixation** | All money amounts in EUR cents (`i64`) | `Money` value object: `amount: i64` (cents), `currency: CurrencyEur`; conversion logic out of scope |
| **Test Coverage** | Minimum 80% unit test coverage; integration tests against real PostgreSQL | `make test` across workspace; CI gate fails if coverage < 80% |
| **Performance** | Auth API P95 < 50ms; Catalog search P95 < 100ms; Checkout P95 < 200ms | Load-tested with concurrent checkouts (≥50 parallel requests) |

## 3. Database Schemas Summary

| Schema | Primary Tables | Key Purpose |
|---|---|---|
| **`auth`** | `users`, `schema_migrations` | Identity management, authentication, password hashing |
| **`catalog`** | `products`, `product_variants`, `schema_migrations` | Product listings, organic certifications, variant attributes |
| **`sales`** | `orders`, `order_items`, `schema_migrations` | Order lifecycle, cart management, price snapshotting |
| **`inventory`** | `batches`, `schema_migrations` | Batch tracking, origin/harvest metadata, stock reservations |

**Core Entity-Relationship Mapping:**
- `auth.users` → 1:N `sales.orders` (user places orders)
- `catalog.products` → 1:N `inventory.batches` (product has batches)
- `catalog.products` → N:1 `sales.orders` (product appears in orders)
- `inventory.batches` → N:1 `sales.orders` (order linked to batch)
- `sales.orders` → N:1 `auth.users` (order belongs to user)

## 4. Active Milestone & Implementation Backlog

| Milestone | Phase | Start Date | Target Completion | Status |
|---|---|---|---|---|
| **M1** | Spec & Foundations | 2026-11-05 | 2026-11-20 | ✅ Complete (brief.md, prd.md, architecture.md, epics-and-stories.md) |
| **M2** | Cargo Workspace & Scaffolding | 2026-11-21 | 2026-12-04 | 🟡 In-Progress (Crates: domain, application, infrastructure, api) |
| **M3** | Auth & Identity Context | 2026-12-05 | 2026-12-18 | ⏳ Pending |
| **M4** | Product Catalog Context | 2026-12-19 | 2027-01-02 | ⏳ Pending |
| **M5** | Sales & Orders Context | 2027-01-03 | 2027-01-16 | ⏳ Pending |
| **M6** | Inventory & Batch Context | 2027-01-17 | 2027-01-30 | ⏳ Pending |
| **M7** | DevOps & Integration | 2027-01-31 | 2027-02-13 | ⏳ Pending |

**Acceptance Criteria Targets:**
- `make db-up` → PostgreSQL running with 4 schemas; `make init-db` applies `init.sql`
- `make test` → all workspace tests pass with ≥80% coverage
- `make build --release` → all 4 crates compile without warnings
- API `/health` endpoint returns 200 JSON `{status: "ok"}`
- `POST /auth/login` returns JWT access token + refresh token
- `GET /catalog/products` returns paginated results with filter support

## 5. Command Interface

| Command | Description | Example |
|---|---|---|
| `make dev` | Start API server with hot-reload (cargo watch) + PostgreSQL via Docker | `make dev` |
| `make build` | Build all crates in release mode | `make build` |
| `make test` | Run all tests across workspace (`cargo test --workspace`) | `make test` |
| `make db-up` | Start PostgreSQL Docker container | `make db-up` |
| `make db-down` | Stop PostgreSQL Docker container | `make db-down` |
| `make init-db` | Initialize schemas (`auth`, `catalog`, `sales`, `inventory`) via `init.sql` | `make init-db` |
| `make migrate-up` | Apply pending SQLx migrations (order: auth → catalog → sales → inventory) | `make migrate-up` |
| `make migrate-down` | Rollback last applied migration | `make migrate-down` |
| `make docs` | Regenerate documentation from current spec | `make docs` |