# ADR-0003: PostgreSQL as the Authoritative Data Store

- **Status:** Proposed
- **Date:** 2026-11-03
- **Project:** `llm-gateway`
- **Related ADRs:** ADR-0001, ADR-0002

## 1. Context

The LLM Gateway manages operational and financial information across multiple tenants.

Its persistent data includes:

- Tenants, projects, users, and API keys.
- Policies and access-control configurations.
- Provider and model registry information.
- Model pricing and capability metadata.
- Inference requests and executions.
- Provider attempts and routing decisions.
- Budget accounts and reservations.
- Token usage and financial ledger entries.
- Audit events.
- Transactional outbox events.
- Evaluation metadata and results.

Several data categories require strong consistency.

For example, two concurrent inference requests must not both consume the same remaining budget. A successful provider execution must not lose its financial usage record because a process crashes.

The gateway therefore needs a durable database that supports transactions, constraints, concurrent updates, relational queries, and reliable recovery.

## 2. Decision Drivers

The persistence architecture must provide:

1. ACID transactions.
2. Strong consistency for financial operations.
3. Referential integrity.
4. Tenant isolation.
5. Concurrent reservation safety.
6. Idempotent settlement.
7. Append-only financial history.
8. Durable event publication.
9. Auditable configuration changes.
10. Schema migration support.
11. Operational maturity.
12. Integration with Rust and SQLx.

## 3. Considered Options

### Option A: PostgreSQL

Advantages:
- ACID transactions.
- Row-level locking.
- Referential integrity.
- JSONB support.
- Strong indexing capabilities.
- Mature migration tooling.
- Row-level security.
- Reliable transactional outbox implementation.

Disadvantages:
- Requires database operations and maintenance.
- High-volume analytical queries require careful workload separation.
- Horizontal write scaling adds complexity.

### Option B: MongoDB

Advantages:
- Flexible document schema.
- Natural JSON representation.
- Convenient for rapidly changing document structures.

Disadvantages:
- Relational financial workflows require careful transaction design.
- Referential integrity is less natural.
- The domain has substantial relational relationships.

### Option C: Distributed SQL Database

Advantages:
- Distributed transactions.
- Horizontal scalability.
- Relational data model.

Disadvantages:
- Additional operational complexity.
- Unnecessary for the initial deployment.
- More difficult local development and testing.

### Option D: Multiple Databases by Capability

Advantages:
- Independent scaling and storage choices.
- Workload isolation.

Disadvantages:
- Distributed consistency complexity.
- More operational dependencies.
- Harder financial reconciliation.

## 4. Decision

Adopt **PostgreSQL as the authoritative transactional database**.

PostgreSQL owns all durable business and financial records.

Redis may support coordination, caching, and short-lived operational state, but it must not become the authoritative source for financial accounting or other durable business records.

The initial deployment will use one PostgreSQL instance with logically separated schemas.

## 5. Data Ownership

| Data category | Authoritative storage |
|---|---|
| Identity and API keys | PostgreSQL |
| Tenant and project data | PostgreSQL |
| Policies and policy versions | PostgreSQL |
| Provider and model registry | PostgreSQL |
| Model pricing | PostgreSQL |
| Inference executions | PostgreSQL |
| Provider attempts | PostgreSQL |
| Routing decisions | PostgreSQL |
| Budget accounts | PostgreSQL |
| Budget reservations | PostgreSQL |
| Usage records | PostgreSQL |
| Financial ledger | PostgreSQL |
| Audit events | PostgreSQL |
| Transactional outbox | PostgreSQL |
| Evaluation metadata | PostgreSQL |
| Exact cache entries | Redis |
| Rate-limit counters | Redis |
| Distributed concurrency permits | Redis |
| Optional semantic vectors | pgvector or Qdrant, subject to a later decision |

## 6. Logical Schema Organization

Use PostgreSQL schemas to organize ownership.

```text
PostgreSQL
│
├── identity
│   ├── users
│   └── api_keys
│
├── tenancy
│   ├── tenants
│   └── projects
│
├── policy
│   ├── policies
│   └── policy_versions
│
├── registry
│   ├── providers
│   ├── models
│   └── model_pricing
│
├── inference
│   ├── requests
│   ├── executions
│   ├── provider_attempts
│   └── routing_decisions
│
├── accounting
│   ├── budget_accounts
│   ├── budget_periods
│   ├── budget_reservations
│   ├── usage_records
│   └── financial_ledger_entries
│
├── audit
│   └── audit_events
│
├── events
│   └── outbox_events
│
└── evaluation
    ├── evaluation_datasets
    ├── evaluation_runs
    └── evaluation_results
```

Schema boundaries are logical ownership boundaries, not separate databases.

## 7. Transactional Requirements

### 7.1 Budget Reservation

Budget reservation must be atomic.

The operation must:

1. Identify applicable budget accounts.
2. Lock relevant budget rows in deterministic order.
3. Calculate available funds.
4. Verify sufficient available funds.
5. Create a reservation.
6. Update reservation accounting.
7. Commit the transaction.

Conceptually:

\[
Available = Limit - ActualSpend - ActiveReservations
\]

A reservation must not be created if the available amount is insufficient.

Concurrent requests must not exceed the budget through race conditions.

### 7.2 Usage Settlement

Settlement must be idempotent.

Requirements:

- One authoritative settlement per execution.
- Unique settlement identity.
- Accurate actual cost recording.
- Append-only financial ledger.
- Corrections through compensating entries.
- Reconciliation for uncertain provider outcomes.

A provider timeout does not prove that the provider failed to execute the request.

The reservation must remain protected until the outcome can be reconciled according to the execution lifecycle.

### 7.3 Transactional Outbox

Business transactions that must publish events will use the transactional outbox pattern.

For example:

```text
BEGIN TRANSACTION

1. Update inference execution.
2. Record usage.
3. Insert financial ledger entry.
4. Insert outbox event.

COMMIT
```

An asynchronous publisher reads committed outbox events and publishes them to the event transport.

Consumers must be idempotent because event delivery may occur more than once.

## 8. Concurrency and Locking

PostgreSQL transactions must protect financial invariants.

Use:

- `SELECT ... FOR UPDATE` for reservation-critical rows.
- Unique constraints for idempotency.
- Foreign keys for ownership relationships.
- Check constraints for valid financial values.
- Deterministic lock ordering to reduce deadlocks.
- Appropriate transaction isolation for each operation.

Do not use application-level read-then-write logic as a substitute for database concurrency control.

Retry deadlocked or serialization-failed transactions only within bounded policies.

## 9. Tenant Isolation

Tenant isolation must be enforced at multiple levels.

### Application level

Every tenant-scoped query must include the authorized tenant context.

The client must never be trusted to supply an unrestricted tenant identifier.

### Database level

Use PostgreSQL Row-Level Security where appropriate.

Example:

```sql
ALTER TABLE inference.executions
ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_execution_isolation
ON inference.executions
USING (
    tenant_id = current_setting('app.tenant_id', true)::uuid
);
```

Set tenant context transaction-locally:

```sql
SELECT set_config('app.tenant_id', $1, true);
```

Important implementation requirements:

- Use a non-superuser application database role.
- Do not use a role with `BYPASSRLS` for ordinary application queries.
- Ensure connection-pool transactions set and clear tenant context safely.
- Test cross-tenant access explicitly.
- Use separate privileged roles for migrations and controlled administrative operations.

RLS is defense in depth, not a replacement for application authorization.

## 10. Financial Data Integrity

Financial data requires stronger controls than ordinary operational data.

Requirements:

1. Use PostgreSQL `NUMERIC` for monetary values.
2. Never use floating-point arithmetic for authoritative financial calculations.
3. Store currency explicitly.
4. Version model pricing.
5. Record pricing version used for each cost calculation.
6. Keep financial ledger entries immutable.
7. Record actual provider usage separately from estimates.
8. Make settlement idempotent.
9. Preserve uncertain execution outcomes.
10. Provide reconciliation jobs and audit history.

A budget overrun caused by an actual provider charge must be recorded accurately. The system must not suppress real expenditure to preserve a budget limit.

## 11. Data Types and Conventions

| Concept | PostgreSQL representation |
|---|---|
| Entity identifiers | UUID |
| Money | NUMERIC |
| Currency | CHAR(3) or controlled currency code |
| Token counts | BIGINT |
| Boolean flags | BOOLEAN |
| Timestamps | TIMESTAMPTZ |
| Structured metadata | JSONB |
| Status values | Controlled TEXT or PostgreSQL ENUM |
| Idempotency keys | TEXT with appropriate unique constraints |
| Version numbers | BIGINT or INTEGER |

Use UTC timestamps.

Use explicit precision and scale for monetary columns based on supported currencies and pricing requirements.

Do not store secrets in ordinary JSONB configuration records.

## 12. SQLx Integration

Use SQLx for asynchronous PostgreSQL access.

Responsibilities:

- Connection pooling.
- Parameterized queries.
- Compile-time checked queries where practical.
- Transaction management.
- Database migrations.
- Explicit row-to-domain mapping.

Example repository:

```rust
pub struct PostgresBudgetRepository {
    pool: sqlx::PgPool,
}

impl PostgresBudgetRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}
```

The repository implements the application-owned `BudgetRepository` interface.

SQLx models must not leak into domain entities.

## 13. Migrations

Use version-controlled SQL migrations.

Proposed structure:

```text
migrations/
├── 0001_create_identity_schema.sql
├── 0002_create_tenancy_schema.sql
├── 0003_create_policy_schema.sql
├── 0004_create_registry_schema.sql
├── 0005_create_inference_schema.sql
├── 0006_create_accounting_schema.sql
├── 0007_create_audit_schema.sql
├── 0008_create_events_schema.sql
└── 0009_create_evaluation_schema.sql
```

Migration requirements:

- Migrations are deterministic and versioned.
- Applied migrations are not edited retroactively.
- Schema changes are reviewed.
- Production changes follow expand-and-contract where required.
- Destructive migrations require explicit rollout planning.
- Migration execution uses a dedicated database role.

## 14. Indexing and Query Performance

Initial indexing principles:

- Index tenant-scoped query columns.
- Index execution IDs and provider attempt IDs.
- Index reservation status and expiry.
- Index ledger account and creation timestamp.
- Index idempotency keys.
- Index outbox publication status and creation timestamp.
- Use composite indexes for common tenant/time queries.
- Avoid unnecessary JSONB indexing.
- Review query plans for high-volume endpoints.

Partition large append-heavy tables such as usage records and audit events only when measured volume justifies it.

Do not introduce partitioning prematurely.

## 15. Retention and Archival

Retention must respect financial, security, and privacy requirements.

Initial policy principles:

- Financial ledger: retain according to applicable legal and contractual requirements.
- Usage records: configurable retention and archival.
- Audit events: controlled retention with tamper-resistant storage requirements.
- Inference content: minimize collection and retain only when explicitly required.
- Outbox events: purge only after successful publication and retention expiry.
- Evaluation records: retain dataset and evaluator version references.

Exact retention periods will be defined in a later operational and compliance decision.

## 16. Backup and Recovery

PostgreSQL production deployment must support:

- Automated backups.
- Point-in-time recovery.
- Backup encryption.
- Restricted backup access.
- Restore testing.
- Documented recovery procedures.

Initial planning targets:

| Measure | Target |
|---|---:|
| Recovery Point Objective | 15 minutes |
| Recovery Time Objective | 1 hour |

These are planning objectives, not verified capabilities. They must be validated through restore and disaster-recovery exercises.

## 17. Testing Requirements

### Unit and integration tests

- Repository behavior.
- Transaction rollback.
- Constraint enforcement.
- Tenant isolation.
- RLS behavior.
- Migration correctness.

### Financial concurrency tests

- Concurrent budget reservations.
- Duplicate settlement.
- Settlement after timeout.
- Reservation expiry.
- Ledger immutability.
- Actual expenditure exceeding reservation.
- Deadlock and serialization retry.

### Outbox tests

- Atomic business update and event creation.
- Publisher restart.
- Duplicate event delivery.
- Consumer idempotency.
- Failed publication recovery.

### Operational tests

- Connection pool exhaustion.
- Database outage.
- Backup restoration.
- Migration rollback or forward recovery.

## 18. Consequences

### Positive

- Strong financial consistency.
- Durable business records.
- Referential integrity.
- Clear ownership of persistent data.
- Mature Rust integration through SQLx.
- Reliable transactional outbox support.

### Negative

- Requires careful schema and transaction design.
- Database contention must be monitored.
- Large analytical workloads may need separate read or analytics infrastructure.
- Horizontal write scaling may eventually require architectural changes.

### Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Budget race conditions | Atomic reservations and row locking |
| Duplicate financial settlement | Unique execution-based settlement keys |
| Lost event publication | Transactional outbox |
| Cross-tenant data exposure | Application authorization and RLS |
| Database overload | Pool limits, indexes, workload management |
| Migration failure | Versioned migrations and rollout strategy |
| Financial record tampering | Append-only ledger and restricted database permissions |

## 19. Validation Criteria

ADR-0003 is validated when:

- PostgreSQL is the authoritative durable data store.
- Core schemas are migrated through SQLx-compatible migrations.
- Financial reservations are concurrency-safe.
- Settlement is idempotent.
- Ledger entries are append-only.
- Outbox publication is recoverable.
- Tenant isolation is tested.
- Redis loss does not destroy authoritative financial records.
- Backup and restore procedures are documented and tested.

## 20. Related ADRs

- ADR-0001: Clean Architecture and Dependency Direction
- ADR-0002: Rust Workspace Structure and Crate Boundaries
- ADR-0004: Redis for Distributed Coordination and Caching
- ADR-0005: Transactional Outbox and Event Delivery
- ADR-0009: Budget Reservation and Financial Settlement
- ADR-0022: Tenant Isolation and PostgreSQL RLS
