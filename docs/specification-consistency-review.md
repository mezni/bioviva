# LLM Gateway — Specification Consistency Review

**File:** `docs/specification-consistency-review.md`  
**Version:** 1.0  
**Status:** Draft — Pre-Implementation Review  
**Project:** `llm-gateway`

---

# 1. Purpose

This document defines the consistency checks required before implementation begins.

The project has specifications covering:

- Product requirements.
- System architecture.
- Domain model.
- Data model.
- API contracts.
- Security.
- Routing policies.
- Provider integrations.
- Reliability.
- Rate limiting and concurrency.
- Budget and usage accounting.
- Observability.
- Evaluation.
- Caching.
- Testing.
- Deployment.
- Definition of Done.
- Build plan.

The objective is to ensure that these documents describe one coherent system.

This review does not replace the existing specifications. It identifies decisions that must be reconciled and establishes the implementation baseline.

# 2. Review Status

| Area | Review Status | Required Action |
|---|---|---|
| Product scope | Defined | Validate scope against implementation |
| Clean Architecture | Defined | Establish final crate boundaries |
| Domain model | Existing document | Cross-check domain entities |
| Data model | Requires confirmation | Validate tables and relationships |
| API contracts | Defined | Reconcile with domain and provider types |
| Security | Defined | Validate enforcement boundaries |
| Routing | Defined | Reconcile with model registry |
| Provider integration | Defined | Establish canonical adapter interface |
| Reliability | Defined | Define execution state ownership |
| Admission control | Defined | Reconcile Redis and PostgreSQL responsibilities |
| Budget accounting | Defined | Validate financial invariants |
| Observability | Defined | Define event and trace schemas |
| Evaluation | Defined | Define worker event contracts |
| Cache | Defined | Validate cache eligibility and fingerprint |
| Testing | Defined | Map tests to requirements |
| Deployment | Defined | Validate infrastructure dependencies |
| Build plan | Defined | Validate milestone dependencies |

Status values:

- **Defined:** A specification exists.
- **Requires confirmation:** A decision or artifact must be verified.
- **Requires reconciliation:** Multiple documents must agree.
- **Implementation pending:** Specification is ready, implementation remains outstanding.

# 3. Critical Architecture Decisions

## 3.1 Clean Architecture Boundaries

**Issue:** The proposed workspace contains many logical components. Creating every component as an independent crate could introduce unnecessary complexity and dependency management overhead.

**Decision:** Use meaningful architectural boundaries rather than one crate per feature.

Recommended initial workspace:

```text
crates/
├── gateway-api/
├── gateway-core/
├── domain/
├── infrastructure/
├── provider-core/
├── provider-adapters/
└── shared-kernel/
```

Responsibilities:

| Crate | Responsibility |
|---|---|
| `gateway-api` | Axum handlers, middleware, HTTP and SSE |
| `gateway-core` | Application use cases and orchestration |
| `domain` | Business entities, value objects, domain rules |
| `infrastructure` | PostgreSQL, Redis, event transport, external implementations |
| `provider-core` | Canonical provider interfaces and types |
| `provider-adapters` | OpenAI, Anthropic, Gemini, compatible providers |
| `shared-kernel` | Cross-cutting identifiers and carefully selected shared primitives |

Logical modules inside these crates can later be extracted when independent ownership, dependency constraints, or deployment requirements justify it.

**Acceptance criteria:**

- Domain has no dependency on Axum, SQLx, Redis, or provider SDKs.
- Application use cases depend on abstractions.
- Infrastructure implements those abstractions.
- Provider-specific types remain inside provider integration boundaries.

## 3.2 Provider Architecture

Use three distinct concepts:

1. Provider registry: provider configuration and lifecycle.
2. Provider core: canonical interfaces and normalized types.
3. Provider adapters: provider-specific HTTP and streaming implementations.

Avoid creating competing provider abstractions in multiple crates.

**Decision:** `provider-core` owns the canonical `ProviderAdapter` contract.

## 3.3 Event Architecture

The specifications refer to Redis Streams and a general event broker.

**Decision for initial release:**

- Use PostgreSQL transactional outbox for durable business events.
- Use Redis Streams as the initial event transport if its delivery and recovery requirements are validated.
- Use idempotent consumers.
- Keep the event publishing abstraction independent of Redis Streams.
- Introduce Kafka or another broker only when justified by measured requirements.

Financial records must never depend on successful telemetry export or sampled traces.

# 4. Data Ownership and Source of Truth

Every important piece of data must have one authoritative owner.

| Data | Authority | Supporting Infrastructure |
|---|---|---|
| Tenant and project | PostgreSQL | Redis cache where appropriate |
| Virtual API key hashes | PostgreSQL | Redis for short-lived operational data |
| Authorization policies | PostgreSQL/configuration | In-memory validated policy snapshots |
| Provider configuration | PostgreSQL/configuration | In-memory registry |
| Provider credentials | Secret manager | Secret cache subject to rotation policy |
| Model metadata | Model registry in PostgreSQL/configuration | In-memory registry |
| Pricing | PostgreSQL | In-memory versioned snapshot |
| Budget accounts | PostgreSQL | Redis for operational coordination only |
| Reservations | PostgreSQL | Redis where needed for coordination |
| Financial ledger | PostgreSQL | Backup and reconciliation |
| Rate-limit counters | Redis | Defined recovery/degradation policy |
| Concurrency permits | Redis | Lease and recovery mechanism |
| Exact cache | Redis | Reconstructible cache entries |
| Semantic vectors | Vector store | PostgreSQL metadata and authorization |
| Audit events | PostgreSQL or durable audit store | Export pipeline |
| Traces | Observability backend | OpenTelemetry Collector |
| Evaluation results | PostgreSQL | Python evaluation worker |

**Invariant:** Redis must never become the authoritative financial ledger.

# 5. Inference Execution State

Several specifications refer to request, execution, attempt, reservation, and provider identifiers.

Define their relationship consistently.

| Identifier | Meaning |
|---|---|
| `request_id` | Identifies the incoming gateway API request |
| `execution_id` | Identifies one logical inference execution |
| `attempt_id` | Identifies one provider execution attempt |
| `reservation_id` | Identifies a financial reservation |
| `provider_id` | Identifies the provider |
| `model_id` | Identifies the gateway model |
| `trace_id` | Identifies the distributed trace |

One request may have one logical execution and multiple provider attempts.

A retry or fallback creates another provider attempt, not another unrelated logical execution.

## 5.1 Execution State

Define a single execution state machine.

Proposed states:

```text
Received
  |
  v
Admitted
  |
  v
Reserved
  |
  v
Executing
  |
  +----> Completed
  |
  +----> Failed
  |
  +----> Cancelled
  |
  +----> UnknownOutcome
              |
              v
       ReconciliationPending
              |
              v
       Reconciled
```

The final state machine must distinguish:

- Gateway request lifecycle.
- Provider attempt lifecycle.
- Budget reservation lifecycle.
- Financial settlement lifecycle.

Do not combine all these lifecycles into one status field.

# 6. Admission Control and Budget Coordination

**Issue:** Rate limiting, concurrency control, and budget reservations use different infrastructure and consistency guarantees.

Establish the following execution order:

1. Authenticate API key.
2. Resolve tenant and project.
3. Authorize requested operation.
4. Validate request.
5. Apply abuse protection.
6. Apply rate limits.
7. Acquire concurrency permit.
8. Evaluate applicable policy.
9. Estimate cost.
10. Reserve budget transactionally.
11. Resolve eligible model.
12. Execute provider attempt.

The precise position of policy evaluation and routing must be reconciled with the cost-estimation requirements: model selection may be needed to estimate cost, while hard policy constraints must be evaluated before execution.

Compensating actions must release admission resources safely when a later step fails.

**Required invariants:**

- No provider execution before required admission controls.
- No provider execution before required budget reservation.
- No duplicate reservation for the same idempotent operation.
- No permanent concurrency permit leakage.
- No automatic release of uncertain provider outcomes without reconciliation.

# 7. Provider and Routing Contracts

The provider integration and routing specifications must use the same canonical model metadata.

Required model attributes:

- Gateway model identifier.
- Provider identifier.
- Provider-specific model name.
- Context limit.
- Input and output capabilities.
- Streaming support.
- Tool-calling support.
- Structured output support.
- Pricing reference.
- Availability status.
- Model version or configuration version.
- Data residency attributes where applicable.

## 7.1 Responsibility Separation

| Component | Responsibility |
|---|---|
| Model registry | Model metadata and lifecycle |
| Capability matcher | Technical compatibility |
| Policy engine | Mandatory policy constraints |
| Quality evaluator | Evidence-based quality eligibility |
| Cost estimator | Expected cost |
| Router | Candidate selection |
| Reliability engine | Retry and fallback execution decisions |
| Provider adapter | Provider-specific request execution |

The router must not directly implement provider HTTP calls.

The provider adapter must not make tenant budget or authorization decisions.

# 8. Streaming Consistency

Streaming creates special execution and accounting requirements.

All specifications must agree on these rules:

1. Provider streaming events are normalized before reaching the public API boundary.
2. The gateway tracks whether response bytes have been delivered.
3. Fallback is permitted only before output delivery, subject to policy and execution state.
4. Once output has been delivered, the gateway must not transparently restart generation on another provider.
5. Client disconnect triggers cancellation where supported.
6. Usage accounting must handle partial and missing provider usage metadata.
7. An interrupted stream must not be represented as a successful complete response.
8. Unknown provider execution outcomes must remain explicitly identifiable.

# 9. Security Consistency

The security document must agree with the API, caching, provider, and observability documents.

Validate these boundaries:

| Boundary | Required Control |
|---|---|
| API ingress | Authentication and request validation |
| Application | Authorization and tenant ownership |
| Database | Tenant-scoped access and constraints |
| Cache | Tenant and project isolation |
| Provider configuration | SSRF and endpoint validation |
| Secret access | Secret manager authorization |
| Logs | Sensitive-value redaction |
| Telemetry | Tenant-aware access and data minimization |
| Admin APIs | Separate administrative authentication |
| Event consumers | Authorized, validated, idempotent processing |

## 9.1 Administrative Authentication

The specifications mention virtual API keys for inference and JWT/OIDC for administration.

Preserve this separation.

Inference API keys must not automatically grant administrative access.

# 10. API and Domain Contract Consistency

Verify that API contracts map cleanly to application use cases and domain objects.

Required checks:

- Public request schemas do not expose database entities.
- Provider-specific request fields are normalized or explicitly namespaced.
- Internal execution IDs are not confused with public request IDs.
- Model identifiers map consistently to registry records.
- Error codes are stable and documented.
- Monetary values have a defined decimal representation.
- Streaming event semantics are documented.
- Idempotency behavior is consistent across API and accounting specifications.

The OpenAPI specification must be treated as a version-controlled API contract.

# 11. Observability and Event Consistency

Define distinct concepts:

| Concept | Purpose |
|---|---|
| Trace | Distributed execution diagnostics |
| Metric | Aggregated operational measurement |
| Log | Structured diagnostic record |
| Audit event | Security or administrative record |
| Domain event | Business state transition |
| Evaluation event | Asynchronous quality assessment |

These must not be treated as interchangeable.

**Required rule:** Financial accounting and required audit records cannot depend on trace sampling.

## 11.1 Common Correlation Fields

Where applicable, events should include:

- `request_id`
- `execution_id`
- `attempt_id`
- `tenant_id`
- `project_id`
- `provider_id`
- `model_id`
- `occurred_at`
- `schema_version`

Sensitive prompts, API keys, and provider credentials must not be included by default.

# 12. ADR Numbering Consistency

Earlier specifications proposed ADR identifiers from ADR-001 through ADR-078.

Some proposed decisions may overlap or may not yet exist.

Before implementation:

1. Inspect the actual `docs/adr/` directory.
2. Identify existing ADRs.
3. Remove duplicate proposals.
4. Assign unique sequential identifiers.
5. Give each ADR one decision.
6. Link relevant ADRs from architecture documents.
7. Record superseded decisions explicitly.

Do not assume proposed ADR identifiers are already accepted decisions.

Suggested initial ADR topics:

| Topic | Decision |
|---|---|
| Architecture | Clean Architecture and dependency direction |
| Workspace | Initial Rust crate boundaries |
| Database | PostgreSQL as financial source of truth |
| Coordination | Redis for distributed admission controls |
| Events | Transactional outbox and initial transport |
| Providers | Canonical provider adapter contract |
| Routing | Hard eligibility before candidate ranking |
| Accounting | Atomic reservation and idempotent settlement |
| Streaming | No transparent fallback after delivered output |
| Observability | OpenTelemetry and durable business events |

# 13. Build Plan Dependency Review

The build plan should not be interpreted as a rigid requirement to delay all testing, security, and observability until their named milestones.

Apply these rules:

- Testing starts in M0.
- Security starts before M2 and continues throughout development.
- Observability starts with M1 and expands incrementally.
- Database migrations are introduced with the first persistence feature.
- Provider contract tests are established before multiple provider integrations.
- Financial concurrency tests are introduced with budget reservations.
- Deployment automation starts early enough to validate each milestone.
- Performance testing begins with a baseline before optimization.

This prevents late discovery of foundational engineering defects.

# 14. Specification Review Checklist

## Architecture

- [ ] Clean Architecture boundaries are consistent.
- [ ] Rust crate ownership is defined.
- [ ] Provider interfaces have one canonical owner.
- [ ] Dependency direction is documented.
- [ ] Event transport is selected for the initial release.

## Domain and Data

- [ ] Domain entities and value objects are consistent.
- [ ] Data model maps to domain ownership.
- [ ] Database relationships are validated.
- [ ] Execution state transitions are consistent.
- [ ] Financial ledger ownership is clear.

## APIs and Providers

- [ ] API contracts map to application use cases.
- [ ] OpenAPI schemas are consistent.
- [ ] Provider request and response types are normalized.
- [ ] Streaming behavior is consistent.
- [ ] Provider errors have one canonical taxonomy.

## Security and Operations

- [ ] Authentication and authorization boundaries agree.
- [ ] Secret-management responsibilities are clear.
- [ ] Tenant isolation applies to every data boundary.
- [ ] Redis failure policies are documented.
- [ ] Recovery procedures preserve accounting correctness.

## Testing and Delivery

- [ ] Critical invariants map to tests.
- [ ] Build milestones respect dependencies.
- [ ] ADR identifiers are unique.
- [ ] Acceptance criteria are measurable.
- [ ] Release gates reflect actual implementation.

# 15. Review Exit Criteria

The specification review is complete when:

1. No unresolved architectural contradiction remains in the initial implementation scope.
2. Domain and database models agree.
3. API and provider contracts agree.
4. Execution and financial state ownership is defined.
5. Security controls are assigned to explicit boundaries.
6. Event schemas and delivery semantics are defined.
7. Initial Rust workspace boundaries are approved.
8. ADR identifiers are reconciled.
9. Build-plan dependencies are validated.
10. All unresolved non-blocking questions are recorded with owners or future decision points.

# 16. Implementation Authorization

Implementation may begin once the following are approved:

- Product scope.
- Architecture boundaries.
- Domain and data model.
- API contract.
- Security baseline.
- Provider abstraction.
- Budget and accounting invariants.
- Initial deployment approach.

Features outside the approved baseline may remain deferred.

---

# 17. Related Documents

- `docs/product.md`
- `docs/architecture.md`
- `docs/domain-model.md`
- `docs/data-model.md`
- `docs/api-contracts.md`
- `docs/security.md`
- `docs/routing-policy.md`
- `docs/provider-integration.md`
- `docs/reliability.md`
- `docs/rate-limiting-and-concurrency.md`
- `docs/budget-and-usage-accounting.md`
- `docs/observability.md`
- `docs/evaluation-strategy.md`
- `docs/cache-strategy.md`
- `docs/test-strategy.md`
- `docs/deployment.md`
- `docs/definition-of-done.md`
- `docs/build-plan.md`

