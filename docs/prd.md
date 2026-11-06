# BioViva - Product Requirements Document

## Document Header & Metadata

| Field | Value |
|---|---|
| **Product Name** | BioViva |
| **Author** | John, Lead Product Manager |
| **Status** | Draft v0.1.0 |
| **Version** | 0.1.0 |
| **Target Completion Date** | 2027-03-01 |
| **Sprint Length** | 2 weeks |
| **Repository** | /home/dali/WORK/bioviva |
| **Last Updated** | 2026-11-05 |

---

## Product Overview & Strategic Goals

### Mission Statement
To empower health-conscious consumers and artisanal producers with a transparent, secure, and high-performance e-commerce platform for premium organic, bio-certified products, where every batch is traceable from origin to consumption.

### Value Proposition
- **Transparency**: Full batch traceability (origin, harvest date, acidity/purity metrics) for every product.
- **Certification**: Verified organic standards and certifying bodies displayed at point of sale.
- **Artisanal Origin**: Direct connection between producer and consumer, preserving craft quality.
- **Seamless Purchasing**: High-performance checkout built on Rust/Axum/Tokio with sub-50ms auth latency.

### Core KPIs & Success Metrics

| KPI | Target | Measurement |
|---|---|---|
| Checkout Success Rate | ≥ 95% | Orders successfully placed without payment failure |
| Auth API Latency | < 50 ms | P95 response time for JWT issuance/validation |
| Product Catalog Accuracy | 100% | All products have required organic certification attributes |
| Batch Traceability Coverage | ≥ 90% | Products with origin/harvest metadata captured |
| System Uptime | ≥ 99.9% | Monthly availability metrics |
| SQL Injection Prevention | 0 incidents | Security audit zero-findings target |

---

## User Personas & Core Journeys

### Persona A: Premium Bio Consumer (End User)
- ** demographics**: Health-aware, 25-55, values sustainability and transparency.
- **goals**: Discover verified organic products; verify batch origin and purity; complete secure purchase.
- **key journey**: 
  1. Lands on homepage → filters by certification (e.g., "EU Organic", "Demeter").
  2. Views product page → reads batch traceability panel (origin farm, harvest date, acidity).
  3. Adds product to cart → proceeds to checkout → enters shipping details → receives order confirmation with batch ID.
  4. Tracks order → views origin details via unique batch number.

### Persona B: Platform Administrator / Inventory Manager
- **demographics**: Ops team, 30-55, manages product lifecycle and order fulfillment.
- **goals**: Maintain catalog integrity; monitor stock levels; resolve batch discrepancies; ensure compliance.
- **key journey**:
  1. Logs in via admin portal → navigates to catalog management.
  2. Reviews pending product submissions from suppliers → validates certification documents.
  3. Adjusts inventory levels → manually reserves stock for incoming orders.
  4. Generates batch traceability reports for regulatory compliance.

### Persona C: Organic Supplier / Artisan Partner
- **demographics**: Small-batch producer, 30-60, owns farm/co-op.
- **goals**: List products with full bio-attributes; manage batch inventory; receive order notifications.
- **key journey**:
  1. Onboards via supplier self-service portal → provides certifying body, certificate number.
  2. Creates product listings → specifies extraction method, acidity range, harvest date, origin region.
  3. Ships orders → inventory auto-decrements; supplier notified of order fulfillment.
  4. Views dashboard → sees sales analytics and batch stock levels.

---

## Bounded Contexts & Functional Requirements (Epics & User Stories)

### Epic 1: Auth & Identity Context
**Goal**: Secure authentication, authorization, and identity management for all three user roles.

| ID | User Story | Acceptance Criteria (Given/When/Then) |
|---|---|---|
| AS-001 | As a consumer, I want to register with email/password so I can access the platform. | **Given** I am on the registration page, **when** I enter valid email and password meeting minimum complexity, **then** my account is created and I am automatically logged in. |
| AS-002 | As a consumer, I want to log in with my credentials so I can access my account. | **Given** I am on the login page with valid email/password, **when** I submit the form, **then** a JWT is issued (signed with Argon2-derived key) and I am redirected to the home page. |
| AS-003 | As an admin, I want role-based access control so that I can only access admin endpoints. | **Given** a JWT is presented to a protected admin endpoint, **when** the token role claim is not "admin", **then** the request returns 403 Forbidden. |
| AS-004 | As a user, I want a password reset flow so I can regain access if I forget my password. | **Given** I click "Forgot Password" with my registered email, **when** the reset link is generated and clicked with a one-time token, **then** I can set a new password (Argon2 hashed). |
| AS-005 | As the system, I want to validate JWT tokens on every protected request so that unauthorized access is blocked. | **Given** a request with an expired/invalid JWT, **when** the token is verified, **then** the request returns 401 Unauthorized. |

### Epic 2: Catalog Context
**Goal**: Product management with bio-certification attributes, custom variants, and discoverability.

| ID | User Story | Acceptance Criteria (Given/When/Then) |
|---|---|---|
| CAT-001 | As a supplier, I want to create a product listing with organic certification attributes so consumers can verify authenticity. | **Given** I am authenticated as a supplier, **when** I provide product name, organic standard, certifying body, certificate number, extraction method, acidity range, harvest date, origin region, **then** the product is persisted in the `catalog` schema with all attributes. |
| CAT-002 | As a consumer, I want to filter products by organic certification and acidity range so I can find products matching my preferences. | **Given** I apply filters for organic standard (e.g., "EU Organic") and acidity (e.g., 0.5-0.8%), **when** the search is executed, **then** only products matching all filter criteria are returned. |
| CAT-003 | As a system, I want to support custom product variants (weight, size) so that consumers can select the desired quantity. | **Given** a product has variant options defined, **when** I select a variant at checkout, **then** the correct price and stock are applied per variant. |
| CAT-004 | As an admin, I want to search products by certifying body so I can audit certification compliance. | **Given** I search by certifying body name, **when** the query runs, **then** all products with that certifying body are returned. |
| CAT-005 | As the system, I want to ensure price snapshotting so that historical order pricing is preserved. | **Given** an order is created, **when** the order is persisted, **then** the unit price at order time is stored (immutable) and not affected by later price changes. |

### Epic 3: Sales & Orders Context
**Goal**: Cart management, order state machine, checkout, and price snapshotting.

| ID | User Story | Acceptance Criteria (Given/When/Then) |
|---|---|---|
| S-001 | As a consumer, I want to add/remove items from my cart so I can adjust my order before checkout. | **Given** I am on the product page, **when** I click "Add to cart", **then** the item is added to my persistent cart session. |
| S-002 | As a consumer, I want to see a summary of my cart with totals before checkout so I can confirm my order. | **Given** items are in my cart, **when** I view the cart summary, **then** subtotal, taxes, and total are displayed correctly (EUR currency). |
| S-003 | As a consumer, I want to checkout with address and payment info so my order is processed. | **Given** a valid cart with in-stock items, **when** I submit checkout with complete address, **then** an order is created with state `PendingPayment` → `Paid` → `Prepared` → `Shipped`. |
| S-004 | As the system, I want an order state machine so that order progress is tracked and immutable. | **Given** an order exists in `PendingPayment` state, **when** payment is confirmed, **then** the state transitions to `Paid` atomically. |
| S-005 | As the system, I want to reserve batch inventory during checkout so that stock is not oversold. | **Given** an order is in `PendingPayment` state, **when** payment is confirmed, **then** the corresponding batches have their stock decremented atomically. |
| S-006 | As an admin, I want to view order history with batch references so I can trace fulfillment. | **Given** I query orders by user or date range, **when** the results include batch numbers and harvest dates, **then** full traceability is displayed. |

### Epic 4: Inventory & Batch Context
**Goal**: Stock reservations, batch tracking by origin/harvest date, and race-condition-safe decrements.

| ID | User Story | Acceptance Criteria (Given/When/Then) |
|---|---|---|
| IB-001 | As a supplier, I want to create a batch with origin and harvest date so that traceability is established. | **Given** I am authenticated, **when** I provide batch number, origin farm, harvest date, purity %, acidity %, **then** the batch is stored in the `inventory` schema and linked to the product. |
| IB-002 | As the system, I want to decrement batch stock atomically during order fulfillment so that race conditions are prevented. | **Given** an order requires N units from batch B, **when** the order state transitions to `Paid`, **then** batch B's available stock is reduced by N using a PostgreSQL `SELECT ... FOR UPDATE` within a transaction. |
| IB-003 | As a consumer, I want to view batch traceability info on the product page so I know the origin and quality metrics. | **Given** a product page is loaded, **when** batch information exists, **then** origin farm, harvest date, purity, and acidity are displayed. |
| IB-004 | As the system, I want to prevent allocation of expired batches so that only viable stock is reserved. | **Given** a batch has a harvest date older than 12 months (configurable), **when** a checkout attempt uses that batch, **then** the request is rejected and an alternative batch is suggested. |
| IB-005 | As an admin, I want to generate batch stock reports so I can audit inventory health. | **Given** I request a batch inventory report, **when** the report lists each batch with current stock, origin, and harvest date, **then** the report is exportable as CSV. |

---

## Non-Functional Requirements (NFRs)

### Security
- **Password hashing**: All passwords must be hashed using Argon2id with recommended parameters (memory=64MiB, time=3, parallelism=2).
- **Token expiration**: JWT access tokens valid for 15 minutes; refresh tokens valid for 7 days with rotation.
- **SQL injection prevention**: All queries must use SQLx parameterized queries; no string concatenation for SQL values.
- **HTTPS-only**: All external communications must enforce TLS 1.3.
- **RBAC enforcement**: Role claims in JWT must be validated at the Axum middleware level.

### Performance
- **API latency**: 
  - Auth endpoints: P95 < 50 ms 
  - Catalog search: P95 < 100 ms 
  - Checkout: P95 < 200 ms
- **Concurrent connections**: Rust/Tokio server must handle ≥ 10,000 simultaneous WebSocket/HTTP connections.
- **Database query optimization**: Primary key lookups ≤ 5 ms; indexed fields on `batch_number`, `certifying_body`, `harvest_date`.

### Reliability & Data Integrity
- **Database transaction isolation**: Use `READ COMMITTED` isolation level with explicit `FOR UPDATE` locks for batch decrements.
- **Schema separation**: Tables physically separated into `auth`, `catalog`, `sales`, `inventory` schemas; cross-schema joins only via documented read models.
- **Backup & recovery**: Daily PostgreSQL snapshots; point-in-time recovery (PITR) enabled.
- **Graceful degradation**: If inventory DB is unavailable, catalog reads remain functional; checkout fails with clear error, not silent data loss.
- **Idempotent operations**: Key checkout operations (order creation, batch reservation) must be idempotent on retry.

---

## Data Contract & Domain Entity Model Highlights

### Core Aggregates

| Aggregate | Key Fields | Value Objects |
|---|---|---|
| **Product** | `id`, `name`, `description`, `price`, `status` | `Certification` (standard, body, number), `Variant` (weight, size), `Dimensions` |
| **Order** | `id`, `user_id`, `state`, `total`, `created_at`, `batch_id` | `Money` (amount, currency=EUR), `Address`, `OrderState` (PendingPayment, Paid, Prepared, Shipped, Cancelled) |
| **Batch** | `id`, `product_id`, `origin_farm`, `harvest_date`, `purity_percent`, `acidity_percent`, `stock` | `Origin` (farm, region), `QualityMetrics` (purity, acidity) |
| **User** | `id`, `email`, `password_hash`, `role`, `created_at` | `Name`, `Address` (shipping/billing) |

### Key Invariants
- **Price snapshotting**: `Order.total` is computed from `Product.price` at order creation and stored immutably.
- **Batch stock non-negative**: `Batch.stock` ≥ 0 at all times; decrement operations guard against underflow.
- **EUR currency fixation**: All `Money` values must have `currency = "EUR"`; conversion logic is out of scope.
- **JWT signing**: Access tokens signed with HMAC-SHA256 using a key derived from Argon2 hash of user password.

---

## Risk Analysis & Mitigation Plan

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| **Inventory race conditions during checkout** | Overselling; customer dissatisfaction | Medium | Use `SELECT ... FOR UPDATE` within a single transaction; test with concurrent load (≥ 50 parallel checkout requests). |
| **Expired batch allocations** | Shipping outdated product; compliance violation | Low | Enforce harvest date validation (≥ 12-month freshness window); automated batch purge job. |
| **Domain leaks across layers (Infrastructure calling Domain logic)** | Tight coupling; harder testing; future refactor risk | Medium | Enforce strict DDD boundaries: Domain crates contain pure Rust structs/traits; Infrastructure crates implement adapters. |
| **JWT/Argon2 integration misconfiguration** | Weak password hashing; token spoofing | Low | Use vetted crates (`argon2`, `jsonwebtoken`); security review before MVP launch; unit tests for token validation. |
| **Single PostgreSQL bottleneck** | Latency spikes under load; single point of failure | Medium | Plan read-replica addition for Q2; implement query caching for read-heavy catalog operations. |
| **Schema separation oversight** | Accidental cross-schema data corruption | Low | Enforce schema prefix in all SQL (`auth.users`, `catalog.products`, `sales.orders`, `inventory.batches`); CI lint check. |
| **EUR-only currency limitation** | Inability to serve international customers | Low (MVP) | Document as known constraint; future scope for multi-currency in Phase 2. |

*Assumptions: Development team has intermediate Rust async experience; PostgreSQL 13+ with schema support; EUR as fixed transaction currency for MVP timeline.*