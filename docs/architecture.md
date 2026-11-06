# BioViva - Architecture Specification

## 1. Architectural Overview & Design Principles

### Clean Architecture Layer Definitions

The system follows Clean Architecture with four concentric layers, where the innermost layers contain the core business logic and outer layers provide supporting services. The **Dependency Rule** states that source code dependencies can only point inward — nothing in an outer layer can depend on anything in an inner layer.

| Layer | Responsibility | Key Characteristics |
|---|---|---|
| **Domain** (innermost) | Enterprise business rules, pure logic | Zero external IO dependencies; no framework imports; only standard Rust types; contains Entities, Aggregates, Value Objects, Domain Errors |
| **Application** | Use cases, orchestration | Orchestrates Domain objects to deliver specific user outcomes; contains Command/Query handlers, Repository interface traits; depends on Domain only |
| **Infrastructure** | External concerns | Implements Application interfaces using external tools; SQLx PostgreSQL adapters, Argon2 hashing, JWT generation; depends on Application and Domain |
| **API** (outermost) | HTTP interface, entry point | Axum-based HTTP server; route handlers, extractors, middleware; depends on Infrastructure, Application, Domain |

### Core Dependency Rule Enforcement

```
Domain  →  (no outward dependencies)
Application  →  Domain only
Infrastructure  →  Application, Domain
API  →  Infrastructure, Application, Domain
```

- The `domain` crate contains zero `use` statements pointing to any external crate that provides IO or framework functionality.
- All cross-layer communication goes through **trait interfaces** defined in Application and implemented in Infrastructure.
- Domain entities are plain Rust structs with no runtime dependencies; they can be tested in isolation without a database or HTTP server.

---

## 2. Bounded Context Map & PostgreSQL Schema Mapping

### Logical Boundary Breakdown

Four distinct bounded contexts operate within a single shared PostgreSQL instance, each isolated via dedicated schemas:

| Bounded Context | Schema | Primary Responsibility |
|---|---|---|
| **Auth** | `auth` | User identification, authentication, authorization, password hashing |
| **Catalog** | `catalog` | Product listings, organic certification attributes, variant management |
| **Sales** | `sales` | Order lifecycle, cart management, price snapshotting, order state machine |
| **Inventory** | `inventory` | Batch tracking, stock reservations, origin/harvest metadata, purity/acidity metrics |

### SQL Schema Isolation Specs

All schemas reside in one PostgreSQL instance but are physically separated:

```sql
-- Auth schema tables
CREATE TABLE auth.users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(128) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'consumer',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Catalog schema tables
CREATE TABLE catalog.products (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price_eur NUMERIC(12,2) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    certifying_body VARCHAR(255),
    certificate_number VARCHAR(255),
    extraction_method VARCHAR(100),
    acidity_range VARCHAR(50),
    harvest_date DATE,
    origin_region VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Sales schema tables
CREATE TABLE sales.orders (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES auth.users(id),
    state VARCHAR(50) NOT NULL DEFAULT 'PendingPayment',
    total_eur NUMERIC(12,2) NOT NULL,
    batch_id UUID REFERENCES inventory.batches(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    paid_at TIMESTAMPTZ,
    shipped_at TIMESTAMPTZ,
    price_snapshot_eur NUMERIC(12,2) NOT NULL
);

-- Inventory schema tables
CREATE TABLE inventory.batches (
    id UUID PRIMARY KEY,
    product_id UUID NOT NULL REFERENCES catalog.products(id),
    origin_farm VARCHAR(255) NOT NULL,
    harvest_date DATE NOT NULL,
    purity_percent NUMERIC(5,2) NOT NULL,
    acidity_percent NUMERIC(5,2) NOT NULL,
    stock_quantity INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### Entity-Relationship Diagram (Mermaid markdown)

```mermaid
erDiagram
    auth.users ||--o| sales.orders : "places"
    catalog.products ||--o| inventory.batches : "has"
    catalog.products ||--o| sales.orders : "appears-in"
    inventory.batches ||--o| sales.orders : "linked-to"
    
    auth.users {
        UUID id PK
        VARCHAR email UK
        VARCHAR password_hash
        VARCHAR role
        TIMESTAMPTZ created_at
    }
    
    catalog.products {
        UUID id PK
        VARCHAR name
        TEXT description
        NUMERIC price_eur
        VARCHAR status
        VARCHAR certifying_body
        VARCHAR certificate_number
        VARCHAR extraction_method
        VARCHAR acidity_range
        DATE harvest_date
        VARCHAR origin_region
        TIMESTAMPTZ created_at
        TIMESTAMPTZ updated_at
    }
    
    inventory.batches {
        UUID id PK
        UUID product_id FK
        VARCHAR origin_farm
        DATE harvest_date
        NUMERIC purity_percent
        NUMERIC acidity_percent
        INTEGER stock_quantity
        TIMESTAMPTZ created_at
    }
    
    sales.orders {
        UUID id PK
        UUID user_id FK
        UUID batch_id FK
        VARCHAR state
        NUMERIC total_eur
        NUMERIC price_snapshot_eur
        TIMESTAMPTZ created_at
        TIMESTAMPTZ paid_at
        TIMESTAMPTZ shipped_at
    }
```

---

## 3. Rust Cargo Workspace & Crate Boundary Specification

### Complete Directory Layout

```
bioviva/
├── Cargo.toml              # Root workspace manifest
├── Makefile
├── .env.example
├── docker-compose.yml
├── init.sql
├── crates/
│   ├── domain/           # Pure business logic
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── ... (entities, aggregates, value objects, errors)
│   ├── application/      # Use cases, interfaces
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── use_cases.rs
│   │   │   ├── commands.rs
│   │   │   └── queries.rs
│   │   │   └── ports/ (repository traits)
│   ├── infrastructure/   # External implementations
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── db/ (SQLx repos)
│   │   │   │   ├── user_repo.rs
│   │   │   │   ├── product_repo.rs
│   │   │   │   └── batch_repo.rs
│   │   │   ├── auth/ (JWT, Argon2)
│   │   │   └── tracing/ (observability)
│   └── api/              # HTTP entry point
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs   # axum::serve + tokio::run
│       │   ├── routes.rs
│       │   ├── handlers.rs
│       │   ├── extractors.rs
│       │   └── error.rs
│       └── templates/ (optional)
└── docs/
    ├── brief.md
    ├── prd.md
    └── architecture.md
```

### Per-Crate Responsibility Matrix & Dependency Graph

| Crate | Exported Public API | Key Dependencies | Depended Upon By |
|---|---|---|---|
| **domain** | `Entity`, `Aggregate`, `ValueObject`, `DomainError` | (none — zero external) | application, infrastructure, api |
| **application** | `UseCase`, `Command`, `Query`, `Repository` trait interfaces | `domain` | infrastructure, api |
| **infrastructure** | `PgUserRepo`, `PgProductRepo`, `PgBatchRepo`; `Argon2Hasher`; `JwtValidator` | `application`, `domain`, `sqlx`, `tokio`, `jsonwebtoken`, `argon2` | api |
| **api** | `Router`, `axum::Router`, route handlers | `infrastructure`, `application`, `domain`, `axum`, `tokio` | (entry point — `cargo run`) |

### Cargo.toml Dependency Specs (selected)

```toml
# Root workspace Cargo.toml
[workspace]
members = [
    "crates/domain",
    "crates/application",
    "crates/infrastructure",
    "crates/api",
]

# crates/domain/Cargo.toml
[package]
name = "bioivara-domain"
version = "0.1.0"

[dependencies]
# NO external IO or framework dependencies — pure Rust types only

# crates/application/Cargo.toml
[package]
name = "bioivara-application"
version = "0.1.0"

[dependencies]
bioivara-domain = { path = "../domain" }

# crates/infrastructure/Cargo.toml
[package]
name = "bioivara-infrastructure"
version = "0.1.0"

[dependencies]
bioivara-application = { path = "../application" }
bioivara-domain = { path = "../domain" }
sqlx = { version = "0.7", features = ["postgres", "uuid"] }
tokio = { version = "1.38", features = ["full"] }
argon2 = "0.10"
jsonwebtoken = "9.3"
tracing = "0.1"
tracing-subscriber = "0.3"

# crates/api/Cargo.toml
[package]
name = "bioivara-api"
version = "0.1.0"

[dependencies]
bioivara-infrastructure = { path = "../infrastructure" }
bioivara-application = { path = "../application" }
bioivara-domain = { path = "../domain" }
axum = "0.7"
tokio = { version = "1.38", features = ["full"] }
tracing-subscriber = "0.3"
```

### Trait-Based Dependency Injection

- **Repository traits** defined in `application/ports/` (e.g., `UserRepositoryTrait`, `ProductRepositoryTrait`, `BatchRepositoryTrait`).
- **SQLx implementations** in `infrastructure/db/` implement these traits.
- **Axum `Extension`** wires repository instances into request handlers:
  ```rust
  // In application layer
  pub trait UserRepositoryTrait: Send + Sync + 'static {
      async fn find_by_email(&self, email: &str) -> Result<User, DomainError>;
      async fn create(&self, user: &NewUser) -> Result<User, DomainError>;
  }
  
  // In infrastructure layer
  pub struct PgUserRepo {
      pool: sqlx::PgPool,
  }
  impl UserRepositoryTrait for PgUserRepo { ... }
  
  // In API layer / handler
  async fn login_handler(
      Extension(repo): Extension<impl UserRepositoryTrait>,
      Json(payload): Json<LoginPayload>,
  ) -> Result<Json<LoginResponse>, ApiError>
  ```

---

## 4. Layered Implementation Specs & Code Layout Standards

### Domain Layer: Entities, Aggregates, Value Objects

**Domain entities** are plain Rust structs with business invariants encoded in methods:

```rust
// crates/domain/src/lib.rs
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub password_hash: String,  // Argon2 hashed
    pub role: UserRole,
}

pub enum UserRole { Consumer, Producer, Admin }

pub struct Product {
    pub id: uuid::Uuid,
    pub name: String,
    pub certifying_body: Option<String>,
    pub certificate_number: Option<String>,
    pub extraction_method: Option<String>,
    pub acidity_range: Option<String>,  // e.g., "0.5-0.8"
    pub harvest_date: Option<NaiveDate>,
    pub origin_region: Option<String>,
    pub price_eur: Money,
}

pub struct Money {
    pub amount: i64,  // stored in cents (EUR smallest unit)
    pub currency: Currency,
}

pub enum Currency { Eur }

pub struct BatchNumber {
    pub value: String,  // e.g., "BATCH-2027-00123"
    pub product_id: uuid::Uuid,
    pub harvest_date: NaiveDate,
}

pub enum DomainError {
    InvalidEmailFormat,
    PasswordTooWeak,
    InsufficientStock,
    BatchExpired,
    CertificationInvalid,
    OrderCancelled,
}
```

### Application Layer: Use Cases, Command/Query handlers, Repository interface traits

**Repository traits** (example):

```rust
// crates/application/ports/user_repo.rs
use bioivara_domain::{User, DomainError};
use uuid::Uuid;

pub trait UserRepositoryTrait: Send + Sync + 'static {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn create(&self, user: &bioivara_domain::NewUser) -> Result<User, DomainError>;
}

// crates/application/ports/product_repo.rs
pub trait ProductRepositoryTrait: Send + Sync + 'static {
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<catalog::Product>, DomainError>;
    async fn search(&self, filters: catalog::ProductFilters) -> Result<Vec<catalog::Product>, DomainError>;
    async fn create(&self, product: &catalog::NewProduct) -> Result<catalog::Product, DomainError>;
}
```

**Command/Query pattern**:

```rust
// crates/application/use_cases/login.rs
use bioivara_domain::{User, DomainError, NewUser};
use bioivara_application::ports::UserRepositoryTrait;
use argon2::Argon2;

pub struct LoginUser {
    pub email: String,
    pub plain_password: String,
}

pub struct LoginResult {
    pub user: User,
    pub jwt_token: String,
}

pub async fn handle(
    repo: impl UserRepositoryTrait,
    argon2: Argon2<'_>,
    payload: LoginUser,
) -> Result<LoginResult, DomainError> {
    let user = repo.find_by_email(&payload.email)
        .await?
        .ok_or(DomainError::InvalidCredentials)?;
    
    // Verify Argon2 hash
    let argon2 = Argon2::default();
    let is_valid = argon2.verify_password(payload.plain_password.as_bytes(), &user.password_hash)?;
    
    // Generate JWT (moved to infrastructure, but logic here)
    Ok(LoginResult { user, /* jwt token generation delegated */ })
}
```

### Infrastructure Layer: SQLx PostgreSQL repository implementations, Argon2 hashing, JWT token handling

**SQLx repository example**:

```rust
// crates/infrastructure/db/user_repo.rs
use sqlx::PgPool;
use bioivara_domain::User;
use bioivara_application::ports::UserRepositoryTrait;
use argon2::Argon2;
use uuid::Uuid;
use chrono::Utc;

pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl bioivara_application::ports::UserRepositoryTrait for PgUserRepo {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, bioivara_domain::DomainError> {
        let row = sqlx::query_as!(
            User,
            "SELECT id, email, password_hash, role FROM auth.users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| bioivara_domain::DomainError::DbError)?;
        Ok(row)
    }
    
    async fn create(&self, user: &bioivara_domain::NewUser) -> Result<User, bioivara_domain::DomainError> {
        let hash = Aron2::hash_password(user.password.as_bytes(), Some(&argon2::PasswordHasher::default_memory_cost()))
            .map_err(|_| bioivara_domain::DomainError::HashingError)?
            .to_owned();
        
        sqlx::query_as!(
            User,
            "INSERT INTO auth.users (id, email, password_hash, role) VALUES ($1, $2, $3, 'consumer') RETURNING *",
            user.id, user.email, hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| bioivara_domain::DomainError::DbError)
    }
}
```

**JWT token handling**:

```rust
// crates/infrastructure/auth/jwt.rs
use jsonwebtoken::{EncodingKey, DecodingKey, TokenData, encode, decode, Validation};
use chrono::{Utc, Duration};
use bioivara_domain::User;

const JWT_SECRET: &[u8; 64] = b"bioviva-secret-must-be-at-least-32-bytes-long";
const ACCESS_TOKEN_TTL: i64 = 900;  // 15 minutes
const REFRESH_TOKEN_TTL: i64 = 604800;  // 7 days

#[derive(serde::Serialize)]
pub struct Claims {
    pub sub: uuid::Uuid,      // user id
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }
    
    pub fn generate_access_token(&self, user_id: uuid::Uuid, role: &str) -> String {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            role: role.to_string(),
            exp: (now + chrono::Duration::minutes(15)).timestamp(),
            iat: now.timestamp(),
        };
        encode(&jsonwebtoken::Header::default(), &claims, &self.encoding_key).unwrap()
    }
    
    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
        let mut validation = Validation::default();
        validation.validate_exp = true;
        validation.validate_nbf = true;
        decode::<Claims>(token, &self.decoding_key, &validation)
    }
}
```

### API Layer: Axum route handlers, extractors, global error handling, tracing/middleware setup

**Axum router setup**:

```rust
// crates/api/src/main.rs
use axum::{
    Router,
    routing::{get, post},
    middleware,
    Json,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::BroadLayer, EnvFilter};

use crate::infrastructure::jwt::JwtService;
use crate::application::use_cases;

fn create_router(jwt: JwtService) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(handler::auth::register))
        .route("/auth/login", post(handler::auth::login))
        .route("/products", get(handler::product::list))
        .route("/products", post(handler::product::create))
        .route("/cart", post(handler::cart::add))
        .route("/orders", post(handler::order::create))
        .with_state(AppState { jwt })
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn_with_state(jwt.clone(), auth_middleware))
        )
        .layer(EnvFilter::try_from_default_env().unwrap_or_else(|_| "bioivara=info".into()))
}
```

**Global error handling**:

```rust
// crates/api/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, body) = match self {
            ApiError::BadRequest(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized(msg) => (axum::http::StatusCode::UNAUTHORIZED, msg),
            ApiError::NotFound(msg) => (axum::http::StatusCode::NOT_FOUND, msg),
            ApiError::Internal(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, body).into_response()
    }
}

// Extractor for JWT auth
pub async fn auth_extractor(
    Extension(jwt_service): Extension<JwtService>,
    headers: axum::http::HeaderMap,
) -> Result<uuid::Uuid, ApiError> {
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .filter(|h| h.starts_with("Bearer "))
        .ok_or(ApiError::Unauthorized("Missing authorization header".into()))?;
    
    let token = &auth_header[7..]; // strip "Bearer "
    let claims = jwt_service.verify_token(token)
        .map_err(|_| ApiError::Unauthorized("Invalid token".into()))?;
    
    Ok(claims.sub)
}
```

---

## 5. Data Access & Transaction Strategy

### Database Connection Pool Lifecycle (SQLx `PgPool`)

- **Creation**: Single `PgPool` created at API startup via `sqlx::postgres::PgPoolOptions::new().max_connections(20).connect(&database_url).await`.
- **Pool sharing**: The `PgPool` is wrapped in `axum::Extension` and passed downward through application use cases to infrastructure repositories.
- **Lifetime**: Pool lives for the entire application runtime; connections acquired/released per-operation; `SELECT ... FOR UPDATE` used for row-level locking.
- **Health checks**: Periodic `SELECT 1` ping via `pool.acquire().await` to detect disconnects.

### Multi-Schema Database Migrations Strategy

- **Migration tool**: `sqlx-migrate` or `diesel` migrations managed via Makefile.
- **Schema versioning**: Each schema has a `schema_migrations` table tracking applied migrations.
- **Migration files** follow naming: `V1__init_auth.sql`, `V2_add_users_table.sql`, etc.
- **Apply order**: `auth` → `catalog` → `sales` → `inventory` (respects dependency order).
- **Rollback**: Supported per-schema but discouraged; use `down` scripts with data-loss warnings.

Example migration `V1__init_auth.sql`:

```sql
-- Runs against `auth` schema
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(128) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'consumer',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS schema_migrations (
    id SERIAL PRIMARY KEY,
    version VARCHAR(50) NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### Cross-Domain Transactional Consistency (Unit of Work / SQLx transaction passing pattern)

**Unit of Work pattern**: Use a single `PgPool` transaction that spans multiple repository operations during a single logical unit (e.g., checkout).

**Checkout transaction flow**:

```rust
// Application use case level
pub async fn checkout_order(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    cart_items: &[CartItem],
) -> Result<Order, DomainError> {
    // Begin explicit transaction
    let mut tx = pool.begin().await
        .map_err(|_| DomainError::DbTransactionFailed)?;
    
    // 1. Create order in `sales.orders` with `PendingPayment` state
    let order = sqlx::query_as!(
        sales::OrderDto,
        "INSERT INTO sales.orders (user_id, state, total_eur, price_snapshot_eur)
         VALUES ($1, 'PendingPayment', $2, $3)
         RETURNING *",
        user_id,
        total_amount,
        price_snapshot
    )
    .fetch_one(&mut tx)
    .await?;
    
    // 2. Reserve batch stock (atomic decrement with FOR UPDATE)
    for item in &cart_items {
        sqlx::query!(
            "UPDATE inventory.batches SET stock_quantity = stock_quantity - $1 
             WHERE id = $2 AND stock_quantity >= $1
             RETURNING stock_quantity",
            item.quantity,
            item.batch_id
        )
        .fetch_optional(&mut tx)
        .await?;
    }
    
    // 3. Update order state to `Paid`
    sqlx::query!(
        "UPDATE sales.orders SET state = 'Paid', paid_at = now() WHERE id = $1",
        order.id
    )
    .execute(&mut tx)
    .await?;
    
    // Commit transaction
    tx.commit().await
        .map_err(|_| DomainError::DbTransactionFailed)?;
    
    Ok(order)
}
```

**Key invariants**:
- All batch decrements and order creations happen within **one `BEGIN ... COMMIT` block**.
- If any step fails, the entire transaction rolls back → no partial state.
- `SELECT ... FOR UPDATE` on `inventory.batches` prevents race conditions (two concurrent checkouts for same batch).
- Transaction timeout configured at PostgreSQL level (e.g., `lock_timeout = 10000` ms).

---

## 6. Security, Authentication & Session Architecture

### JWT Structure, Claims, Refresh Mechanism

**Access token claims** ( JWT `header` + `payload` + `signature` ):

```json
{
  "sub": "user-uuid-here",
  "role": "consumer|producer|admin",
  "iat": 1730458200,
  "exp": 1730458290  // 15 minutes from iat
}
```

**Refresh token claims**:

```json
{
  "sub": "user-uuid-here",
  "role": "consumer",
  "type": "refresh",
  "iat": 1730458200,
  "exp": 1730544600  // 7 days from iat
}
```

**Token lifecycle**:

1. **Login** → user provides email/password → Argon2 verified → server issues:
   - `access_token` (15 min TTL, `Authorization: Bearer <token>` header)
   - `refresh_token` (7 day TTL, stored HTTP-only cookie or secure client storage)
2. **Access protected routes** → Axum middleware verifies `access_token` claims → `role` RBAC check.
3. **Token expiry** → client receives `401 Unauthorized` → client uses `refresh_token` → new `access_token` issued (no password re-entry).
4. **Refresh token rotation** → each refresh invalidates previous refresh token and issues a new one.
5. **Logout** → refresh token revoked via `revoked_tokens` table or blacklist (JWT `jti` claim).

**Argon2 password hashing pipeline**:

```
Plain password → Argon2id hash (memory=64MiB, time=3 passes, parallelism=2) → stored in `auth.users.password_hash`
Verification:
  1. Retrieve stored hash from DB
  2. Argon2.verify_password(input_password.as_bytes(), &stored_hash)
  3. If valid → proceed (e.g., login, JWT issuance)
  4. If invalid → return authentication error
```

Recommended Argon2 parameters (RFC 9106):
- Memory: 64 MiB
- Time: 3 passes
- Parallelism: 2 threads
- Key length: 32 bytes

### Role-Based Access Control (RBAC) Middleware Flow

**Axum middleware flow** (per-request):

```
HTTP Request
    │
    ▼
Auth extractor reads `Authorization: Bearer <token>` header
    │
    ▼
JWT token validated (signature, expiry, `nbf`)
    │
    ▼
Claims extracted: `sub` (user_id), `role`
    │
    ▼
Lookup `auth.users` by `sub` → verify role matches endpoint requirements
    │
    ▼
✅ If role authorized → handler proceeds (via `Next`)
    │
    ▼
❌ If role NOT authorized → return 403 Forbidden
```

**Middleware implementation sketch**:

```rust
// crates/api/src/middleware/auth.rs
use axum::{
    extract::{FromRequest, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    Json,
};
use serde::Deserialize;
use thiserror::Error;
use jsonwebtoken::{errors::ErrorKind, Decoding, Validation};
use crate::infrastructure::jwt::JwtService;
use crate::application::error::DomainError;

#[derive(Debug, Deserialize)]
pub struct RbacCheck {
    required_role: String,
}

pub async fn auth_middleware<
    S: Send + Sync,
>(
    State(jwt_service): State<JwtService>,
    headers: HeaderMap,
    next: axum::middleware::Next<S>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Extract token
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .filter(|h| h.starts_with("Bearer "))
        .ok_or(ApiError::Unauthorized("Missing bearer token".into()))?;
    
    let token = &auth_header[7..];
    
    // 2. Decode & validate
    let token_data = jwt_service.verify_token(token)
        .map_err(|e| {
            let kind = e.kind();
            if kind == ErrorKind::ExpiredSignature {
                ApiError::Unauthorized("Token expired".into())
            } else {
                ApiError::Unauthorized("Invalid token".into())
            }
        })?;
    
    let claims = token_data.claims;
    
    // 3. Role check (example for admin-only endpoint)
    // This would be passed as middleware parameter in real code
    if claims.role != "admin" {
        return Err(ApiError::Forbidden("Insufficient roles".into()));
    }
    
    // 4. Attach user_id to request extension for handler use
    let user_id = claims.sub;
    
    // 5. Continue to handler
    let response = next.run(request).await;
    Ok(response)
}
```

---

## 7. Scaffolding & CI/CD Tooling Integration

### Environment Configurations (`.env` strategy)

**`.env.example`** (never commit real secrets):

```
# Database
DATABASE_URL=postgres://bioviva:secret@localhost:5432/bioviva?schema=public

# Auth
JWT_SECRET=super-secret-key-at-least-32-bytes-long-for-hmac-sha256
ARGON2_MEMORY=64
ARGON2_TIME=3
ARGON2_PARALLELISM=2

# Server
SERVER_ADDRESS=0.0.0.0:3000
SERVER_SHUTDOWN_GRACE_PERIOD_SECS=15

# Application
APP_NAME=bioivara
API_VERSION=v0.1.0
```

**Per-environment `.env` files**:
- `.env.dev` — local development (localhost DB)
- `.env.staging` — CI/CD staging (managed RDS)
- `.env.prod` — production (managed PostgreSQL, restricted secrets)

### Makefile Task Commands

**Root `Makefile`** (minimum viable scaffolding):

```makefile
.PHONY: all dev build test db-up db-down logs migrate-create migrate-up migrate-down help

all: ## Show this help
	@$(MAKE) help

dev: ## Start development server with hot-reload (via cargo watch)
	@cargo run --package bioivara-api -- --dev

build: ## Build all crates in release mode
	@cargo build --release --all-packages

test: ## Run all tests across workspace
	@cargo test --workspace

db-up: ## Start PostgreSQL via Docker Compose
	@docker-compose up -d db

db-down: ## Stop PostgreSQL Docker container
	@docker-compose down

logs: ## View Docker container logs
	@docker-compose logs -f db

migrate-up: ## Run pending SQL migrations
	@sqlx migrate run --source file://db/migrations

migrate-down: ## Rollback last migration
	@sqlx migrate revert --source file://db/migrations

init-db: ## Initialize schemas (auth, catalog, sales, inventory)
	@psql $(DATABASE_URL) -f init.sql

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "} {printf "%-20s %s\n", $$1, $$2}'
```

**`init.sql`** (schema initialization):

```sql
-- Create schemas if not exist
CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS catalog;
CREATE SCHEMA IF NOT EXISTS sales;
CREATE SCHEMA IF NOT EXISTS inventory;

-- Grant permissions (adjust for your user/role)
GRANT ALL ON SCHEMA auth TO bioviva;
GRANT ALL ON SCHEMA catalog TO bioviva;
GRANT ALL ON SCHEMA sales TO bioviva;
GRANT ALL ON SCHEMA inventory TO bioviva;

-- Grant default privileges on future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA auth GRANT ALL ON TABLES TO bioviva;
ALTER DEFAULT PRIVILEGES IN SCHEMA catalog GRANT ALL ON TABLES TO bioviva;
ALTER DEFAULT PRIVILEGES IN SCHEMA sales GRANT ALL ON TABLES TO bioviva;
ALTER DEFAULT PRIVILEGES IN SCHEMA inventory GRANT ALL ON TABLES TO bioviva;
```

**`docker-compose.yml`** (minimal services):

```yaml
version: "3.9"

services:
  db:
    image: postgres:15-alpine
    container_name: bioviva-db
    environment:
      POSTGRES_USER: bioviva
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-secret}
      POSTGRES_DB: bioviva
    ports:
      - "5432:5432"
    volumes:
      - postgres-data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U bioviva -d bioviva"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  api:
    build: .
    container_name: bioviva-api
    depends_on:
      db:
        condition: service_healthy
    environment:
      DATABASE_URL: postgresql://bioviva:${POSTGRES_PASSWORD:-secret}@db:5432/bioviva?schema=public
      JWT_SECRET: ${JWT_SECRET:-super-secret-key-at-least-32-bytes-long-for-hmac-sha256}
      ARGON2_MEMORY: ${ARGON2_MEMORY:-64}
      ARGON2_TIME: ${ARGON2_TIME:-3}
      ARGON2_PARALLELISM: ${ARGON2_PARALLELISM:-2}
    ports:
      - "3000:3000"
    restart: unless-stopped

volumes:
  postgres-data:
```

This architecture specification provides a production-grade foundation for BioViva, with clear DDD boundaries, PostgreSQL schema isolation, Rust Cargo workspace discipline, and full scaffolding for development, testing, and deployment pipelines.