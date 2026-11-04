# LLM Gateway — Definition of Done

**File:** `docs/definition-of-done.md`  
**Version:** 1.0  
**Status:** Draft  
**Project:** `llm-gateway`  
**Implementation:** Rust + Python  
**Architecture:** Clean Architecture

---

# 1. Purpose

This document defines the Definition of Done (DoD) for every feature, component, milestone, and release in `llm-gateway`.

A feature is not complete merely because its code compiles or its primary workflow works.

It must satisfy the applicable requirements for:

- Functional correctness.
- Architecture.
- Security.
- Data integrity.
- Testing.
- Observability.
- Documentation.
- Operational readiness.

The Definition of Done provides objective evidence that implementation meets the agreed engineering standards.

# 2. Completion Levels

Use four completion levels.

| Level | Name | Meaning |
|---|---|---|
| L1 | Code Complete | Implementation is written and locally validated |
| L2 | Feature Complete | Functional, integration, security, and documentation criteria pass |
| L3 | Release Ready | Release and deployment quality gates pass |
| L4 | Production Ready | Operational, resilience, security, and recovery requirements are validated |

A feature can be L2 without the entire system being L4.

Do not describe the whole gateway as production-ready until its system-level requirements have been demonstrated.

# 3. Universal Definition of Done

Every feature must satisfy the applicable checklist below.

## 3.1 Functional Correctness

- [ ] Acceptance criteria are documented.
- [ ] Expected behavior is implemented.
- [ ] Invalid input is handled.
- [ ] Failure scenarios are handled.
- [ ] Edge cases are considered.
- [ ] Domain invariants are preserved.
- [ ] Error responses are predictable.
- [ ] No known critical functional defects remain.

## 3.2 Architecture

- [ ] Responsibilities are clearly defined.
- [ ] Dependencies follow Clean Architecture boundaries.
- [ ] Domain logic does not depend on infrastructure.
- [ ] External integrations are behind appropriate interfaces.
- [ ] No unnecessary coupling is introduced.
- [ ] No duplicate business rules are introduced.
- [ ] Architectural decisions are documented where needed.
- [ ] Existing ADRs are updated when decisions change.

## 3.3 Code Quality

- [ ] Code follows project conventions.
- [ ] Names communicate intent.
- [ ] Functions and modules have clear responsibilities.
- [ ] Error handling is explicit.
- [ ] No unnecessary `unwrap()` or `expect()` calls in production paths.
- [ ] No unexplained unsafe Rust.
- [ ] No hardcoded secrets.
- [ ] No unnecessary dependencies.
- [ ] No unresolved compiler warnings in required CI checks.

## 3.4 Testing

- [ ] Unit tests pass.
- [ ] Component tests pass.
- [ ] Integration tests pass where applicable.
- [ ] Contract tests pass where applicable.
- [ ] Security tests pass where applicable.
- [ ] Failure paths are tested.
- [ ] Regression tests are included for fixed defects.
- [ ] Tests are deterministic or have documented reasons for controlled nondeterminism.

## 3.5 Security

- [ ] Authentication is enforced where required.
- [ ] Authorization is enforced at the correct boundary.
- [ ] Tenant isolation is preserved.
- [ ] Sensitive information is not logged.
- [ ] Secrets are retrieved securely.
- [ ] Input validation is applied.
- [ ] Abuse and resource exhaustion risks are considered.
- [ ] Security-relevant actions are auditable.

## 3.6 Observability

- [ ] Relevant logs are available.
- [ ] Relevant metrics are emitted.
- [ ] Trace context is propagated where applicable.
- [ ] Errors can be diagnosed.
- [ ] Sensitive data is redacted.
- [ ] Operationally meaningful events are recorded.
- [ ] Dashboards or alerts are updated when required.

## 3.7 Documentation

- [ ] Public interfaces are documented.
- [ ] Configuration changes are documented.
- [ ] Failure behavior is documented.
- [ ] Security implications are documented.
- [ ] Tests and validation commands are documented.
- [ ] Relevant architecture documents are updated.
- [ ] Suggested Git commit message includes `llm-gateway`.

# 4. Architecture Definition of Done

The gateway must preserve its architectural boundaries.

## 4.1 Clean Architecture

- [ ] Domain entities and value objects contain business rules.
- [ ] Application use cases orchestrate domain behavior.
- [ ] Infrastructure implements application-defined interfaces.
- [ ] API handlers do not contain complex business logic.
- [ ] Provider SDK details do not leak into the domain.
- [ ] Database models do not become domain models by default.
- [ ] Dependencies point inward toward stable application and domain abstractions.

## 4.2 Component Boundaries

- [ ] Gateway ingress handles protocol concerns.
- [ ] Identity and authorization enforce access control.
- [ ] Policy engine evaluates applicable policies.
- [ ] Rate limiter controls request and token admission.
- [ ] Concurrency manager controls active executions.
- [ ] Budget manager controls financial reservations and settlement.
- [ ] Model router selects eligible models.
- [ ] Provider adapters normalize provider interactions.
- [ ] Reliability engine controls retry and fallback.
- [ ] Cache engine enforces cache eligibility and isolation.
- [ ] Telemetry captures execution information.
- [ ] Evaluation worker operates asynchronously.

## 4.3 Architectural Invariants

The following invariants must hold:

1. No provider execution before required authentication and authorization.
2. No provider execution before required admission and budget reservation.
3. No cross-tenant data access.
4. No provider credential exposure through ordinary API responses or logs.
5. No duplicate financial settlement for one execution.
6. No retry beyond configured limits.
7. No fallback that violates mandatory model eligibility constraints.
8. No transparent provider fallback after streaming output has been delivered.
9. No evaluation workload that blocks synchronous inference indefinitely.
10. No financial accounting dependency on sampled telemetry.

# 5. Feature-Specific Definition of Done

## 5.1 Identity and API Key Management

- [ ] API keys are generated using cryptographically secure randomness.
- [ ] Raw API keys are returned only at creation or rotation.
- [ ] Only cryptographic hashes or keyed hashes are stored.
- [ ] Expiration and revocation work.
- [ ] Rotation behavior is tested.
- [ ] Key scopes are enforced.
- [ ] Tenant and project ownership are verified.
- [ ] Authentication failures are observable.
- [ ] Sensitive key material is never logged.

## 5.2 Authorization and Tenant Isolation

- [ ] RBAC policies are implemented.
- [ ] Permissions are enforced on every protected operation.
- [ ] Client-supplied tenant identifiers are not trusted as authorization.
- [ ] Database access is tenant-scoped.
- [ ] Cache keys include tenant and applicable project scope.
- [ ] Telemetry access is tenant-aware.
- [ ] Cross-tenant integration tests pass.
- [ ] Administrative actions are audited.

## 5.3 Request Validation

- [ ] Request schemas are validated.
- [ ] Body size limits are enforced.
- [ ] Message and token limits are enforced.
- [ ] Unsupported capabilities are rejected.
- [ ] Invalid model parameters are rejected.
- [ ] Provider-specific fields are normalized or explicitly rejected.
- [ ] SSRF protections apply to configurable endpoints.
- [ ] Errors do not expose internal implementation details.

## 5.4 Rate Limiting and Concurrency

- [ ] Request quotas work at configured scopes.
- [ ] Token quotas use documented estimation and reconciliation rules.
- [ ] Distributed rate limiting is atomic.
- [ ] Concurrency permits are acquired and released safely.
- [ ] Permit recovery handles crashed instances.
- [ ] Limits apply across gateway replicas.
- [ ] Redis failure behavior follows documented policy.
- [ ] Rejection responses contain appropriate retry information.
- [ ] Burst and concurrent-request tests pass.

## 5.5 Budget and Usage Accounting

- [ ] Budget hierarchy is enforced.
- [ ] Reservations are atomic.
- [ ] Concurrent reservations cannot bypass available-budget checks.
- [ ] Pricing is versioned.
- [ ] Unknown pricing follows explicit policy.
- [ ] Settlement is idempotent.
- [ ] Actual provider cost is recorded.
- [ ] Unknown outcomes are reconciled.
- [ ] Financial ledger is auditable.
- [ ] Cache hits are accounted for correctly.
- [ ] Evaluation costs are attributed separately.

## 5.6 Model Routing

- [ ] Model capability matching works.
- [ ] Context-window validation works.
- [ ] Security and residency restrictions are enforced.
- [ ] Provider health influences eligibility.
- [ ] Quality constraints are applied.
- [ ] Pricing data is versioned.
- [ ] Routing decisions are explainable.
- [ ] No eligible model results in a controlled error.
- [ ] Fallback preserves mandatory constraints.
- [ ] Routing behavior is evaluated against a versioned dataset.

## 5.7 Provider Integration

- [ ] Provider request normalization works.
- [ ] Response normalization works.
- [ ] Usage extraction works.
- [ ] Error normalization works.
- [ ] Streaming events are normalized.
- [ ] Timeouts and cancellation are supported.
- [ ] Credentials are retrieved securely.
- [ ] Unsupported capabilities are reported correctly.
- [ ] Provider contract tests pass.
- [ ] Provider configuration is validated before activation.

## 5.8 Reliability

- [ ] Retryable and permanent failures are classified.
- [ ] Retry limits are enforced.
- [ ] Retry-After is respected where applicable.
- [ ] Backoff and jitter are implemented.
- [ ] Circuit breaker transitions are tested.
- [ ] Fallback is constrained by policy.
- [ ] Deadlines are enforced.
- [ ] Cancellation releases resources safely.
- [ ] Unknown provider outcomes are represented explicitly.
- [ ] Failure scenarios are observable.

## 5.9 Cache

- [ ] Exact cache fingerprints are deterministic.
- [ ] Cache keys include all response-affecting inputs.
- [ ] Tenant and project isolation are enforced.
- [ ] TTL and invalidation work.
- [ ] Only eligible responses are cached.
- [ ] Incomplete and failed responses are not cached.
- [ ] Cache failures follow documented degradation behavior.
- [ ] Semantic cache false positives are evaluated before enablement.
- [ ] Cost avoidance is not confused with measured realized savings.

## 5.10 Observability

- [ ] Trace IDs and execution IDs are correlated.
- [ ] Provider attempts are represented.
- [ ] Routing decisions are observable.
- [ ] Token usage and cost are captured.
- [ ] Gateway-added latency is measured.
- [ ] Sensitive prompts and secrets are protected.
- [ ] Metrics avoid uncontrolled cardinality.
- [ ] Sampling does not affect financial accounting.
- [ ] Dashboards and alert rules are validated.

## 5.11 Evaluation Worker

- [ ] Events are consumed idempotently.
- [ ] Evaluation datasets are versioned.
- [ ] Evaluator versions are recorded.
- [ ] Quality metrics are calculated correctly.
- [ ] Judge failures do not produce automatic passing results.
- [ ] Evaluation costs are tracked.
- [ ] Results are traceable to model and prompt versions.
- [ ] Regression thresholds are enforced.
- [ ] Evaluation backlog and failure rates are observable.

# 6. API Definition of Done

Every public API endpoint must:

- [ ] Have a documented request schema.
- [ ] Have a documented response schema.
- [ ] Have documented error codes.
- [ ] Enforce authentication.
- [ ] Enforce authorization.
- [ ] Validate input.
- [ ] Enforce rate and resource limits.
- [ ] Return appropriate HTTP status codes.
- [ ] Include request correlation information.
- [ ] Be represented in OpenAPI.
- [ ] Have unit and integration tests.
- [ ] Have security tests where applicable.
- [ ] Preserve documented compatibility guarantees.

For streaming endpoints:

- [ ] SSE framing is valid.
- [ ] Event ordering is defined.
- [ ] Client cancellation works.
- [ ] Partial failures are handled.
- [ ] No transparent restart occurs after output delivery.

# 7. Database Definition of Done

- [ ] Schema changes are represented by migrations.
- [ ] Constraints enforce required invariants.
- [ ] Indexes support expected query patterns.
- [ ] Tenant ownership is enforced.
- [ ] Financial values use suitable decimal representation.
- [ ] Transactions preserve consistency.
- [ ] Idempotency constraints are enforced.
- [ ] Outbox events are committed transactionally with related changes.
- [ ] Migration compatibility is validated.
- [ ] Backup and restore procedures are tested.

# 8. Performance Definition of Done

Performance claims must be supported by repeatable measurements.

Initial project targets:

| Metric | Target |
|---|---|
| Gateway-added latency p95 | Below 50 ms |
| Exact cache lookup p95 | Below 15 ms |
| Baseline concurrency | 1,000 active requests |
| Availability objective | 99.9% |

These are engineering targets, not measured results.

A performance-sensitive feature is complete when:

- [ ] A baseline measurement exists.
- [ ] Workload assumptions are documented.
- [ ] Test environment is documented.
- [ ] Latency percentiles are measured.
- [ ] Throughput is measured.
- [ ] Resource utilization is measured.
- [ ] No unacceptable regression is introduced.
- [ ] Bottlenecks are documented.
- [ ] Relevant capacity limits are understood.

# 9. Security Definition of Done

- [ ] Threat model reviewed.
- [ ] Authentication and authorization validated.
- [ ] Tenant isolation validated.
- [ ] Secret handling validated.
- [ ] SSRF controls tested.
- [ ] Input validation tested.
- [ ] Rate-limit abuse tested.
- [ ] Sensitive logging reviewed.
- [ ] Dependency vulnerabilities reviewed.
- [ ] Container security reviewed.
- [ ] Audit logging validated.
- [ ] Critical and high-severity unresolved findings are addressed or formally accepted by an authorized owner.

# 10. Documentation Definition of Done

Every completed feature must update relevant documentation.

Potential documents:

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

Documentation must describe actual implemented behavior, not only intended future behavior.

# 11. Pull Request Definition of Done

Every pull request must include:

- [ ] Clear problem statement.
- [ ] Scope of the change.
- [ ] Implementation summary.
- [ ] Relevant architecture decisions.
- [ ] Tests added or updated.
- [ ] Security implications considered.
- [ ] Documentation updated.
- [ ] CI checks passing.
- [ ] No unresolved critical review comments.
- [ ] Suggested commit message includes `llm-gateway`.

Large changes should be split into reviewable increments.

# 12. Release Definition of Done

A release is ready when:

## Functional

- [ ] Release scope is complete.
- [ ] Critical workflows pass.
- [ ] API contracts are validated.
- [ ] Provider contracts are validated.

## Security

- [ ] Security gates pass.
- [ ] No unaccepted critical vulnerabilities remain.
- [ ] Secrets are correctly configured.
- [ ] Access control is validated.

## Reliability

- [ ] Retry and fallback tests pass.
- [ ] Dependency failure behavior is validated.
- [ ] Graceful shutdown is tested.
- [ ] Recovery behavior is validated.

## Quality

- [ ] Regression evaluation passes.
- [ ] Required performance targets are measured.
- [ ] No known critical defects remain.

## Operations

- [ ] Deployment procedure is documented.
- [ ] Health checks work.
- [ ] Dashboards and alerts are available.
- [ ] Backup and restore have been tested.
- [ ] Rollback procedure is documented.
- [ ] Release notes are prepared.

# 13. Production Readiness Definition of Done

Production readiness requires additional operational evidence.

- [ ] SLOs and error budgets are defined.
- [ ] Capacity limits are measured.
- [ ] Incident response procedures exist.
- [ ] On-call ownership is defined.
- [ ] Provider outage procedures exist.
- [ ] Financial reconciliation is operational.
- [ ] Secret rotation is tested.
- [ ] Disaster recovery objectives are validated.
- [ ] Backup restoration is demonstrated.
- [ ] Security review is completed.
- [ ] Tenant isolation is independently tested.
- [ ] Production telemetry access is controlled.
- [ ] Cost controls and alerts are operational.
- [ ] Staging-to-production deployment is repeatable.

A successful local Docker Compose run does not establish production readiness.

# 14. Evidence Requirements

Every milestone should produce evidence.

| Evidence | Example |
|---|---|
| Code | Merged implementation |
| Tests | CI run and test report |
| Security | Scan report and security test results |
| Performance | Benchmark report |
| Evaluation | Dataset and evaluation report |
| Architecture | Updated diagrams and ADRs |
| Operations | Deployment and recovery test |
| Documentation | Reviewed Markdown documents |

Use versioned evidence wherever practical.

# 15. Completion Tracking

Use the following statuses:

| Status | Meaning |
|---|---|
| Not Started | Work has not started |
| In Progress | Implementation underway |
| Code Complete | L1 achieved |
| Feature Complete | L2 achieved |
| Release Ready | L3 achieved |
| Production Ready | L4 achieved |
| Blocked | Depends on unresolved work |
| Deferred | Explicitly excluded from current scope |

Do not mark a feature complete solely because implementation work has finished.

# 16. Architectural Invariants

1. Correctness takes precedence over optimization.
2. Security is part of feature completion.
3. Financial correctness is mandatory.
4. Tenant isolation is mandatory.
5. Failure paths must be tested.
6. Evaluation results must be reproducible.
7. Performance claims require measurements.
8. Documentation must reflect implemented behavior.
9. Production readiness requires operational evidence.
10. Every release must have a rollback strategy.

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
- `docs/build-plan.md`


