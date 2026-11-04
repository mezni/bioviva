# LLM Gateway — Security Architecture & Threat Model

**Version:** 1.0  
**Status:** Draft  
**Project:** `llm-gateway`  
**Architecture:** Clean Architecture  
**Implementation:** Rust + Axum  
**Security approach:** Defense in Depth

---

# 1. Purpose

This document defines the security requirements, threat model, security controls, and operational security practices for the LLM Gateway.

The gateway is a security-sensitive infrastructure component because it:

- Holds access to external LLM providers.
- Processes potentially sensitive prompts and responses.
- Controls customer financial expenditure.
- Manages tenant-specific data.
- Exposes inference endpoints to external applications.
- Integrates with third-party APIs.
- Executes requests on behalf of authenticated customers.

A compromise could result in financial losses, data exposure, unauthorized inference, or abuse of provider credentials.

The security architecture must protect the gateway, its customers, and its infrastructure.

---

# 2. Security Objectives

| ID | Objective | Description |
|---|---|---|
| SEC-001 | Confidentiality | Protect prompts, responses, credentials, and customer data |
| SEC-002 | Integrity | Prevent unauthorized modification of policies, budgets, and configurations |
| SEC-003 | Availability | Protect against abuse and resource exhaustion |
| SEC-004 | Authentication | Verify every protected request |
| SEC-005 | Authorization | Enforce permissions and resource ownership |
| SEC-006 | Tenant isolation | Prevent cross-tenant access |
| SEC-007 | Financial protection | Prevent unauthorized spending |
| SEC-008 | Credential protection | Secure provider and gateway credentials |
| SEC-009 | Auditability | Record security-sensitive operations |
| SEC-010 | Data governance | Enforce retention and privacy requirements |

---

# 3. Security Architecture

```mermaid
flowchart TB
    CLIENT["Client Application"]

    subgraph EDGE["Gateway Security Boundary"]
        TLS["TLS Termination"]
        WAF["Edge Protection"]
        AUTH["Authentication"]
        RBAC["Authorization / RBAC"]
        TENANT["Tenant Isolation"]
        VALIDATION["Request Validation"]
        PII["PII Protection"]
        ABUSE["Abuse Detection"]
        RATE["Distributed Rate Limiter"]
        CONCURRENCY["Concurrency Control"]
        POLICY["Policy Engine"]
        BUDGET["Budget Enforcement"]
    end

    subgraph CORE["Inference Core"]
        ROUTER["Model Router"]
        SECRET["Secret Manager"]
        PROVIDER["Provider Adapters"]
    end

    subgraph DATA["Data Protection"]
        PG[("PostgreSQL")]
        REDIS[("Redis")]
        AUDIT["Audit Logger"]
        KMS["Encryption / KMS"]
    end

    subgraph EXTERNAL["External Providers"]
        OPENAI["OpenAI"]
        ANTHROPIC["Anthropic"]
        LOCAL["Internal Models"]
    end

    CLIENT --> TLS
    TLS --> WAF
    WAF --> AUTH
    AUTH --> RBAC
    RBAC --> TENANT
    TENANT --> VALIDATION
    VALIDATION --> PII
    PII --> ABUSE
    ABUSE --> RATE
    RATE --> CONCURRENCY
    CONCURRENCY --> POLICY
    POLICY --> BUDGET
    BUDGET --> ROUTER
    ROUTER --> SECRET
    SECRET --> PROVIDER

    AUTH --> PG
    RATE --> REDIS
    CONCURRENCY --> REDIS
    BUDGET --> PG
    AUDIT --> PG
    PG --> KMS

    PROVIDER --> OPENAI
    PROVIDER --> ANTHROPIC
    PROVIDER --> LOCAL
```

Security controls must be applied at multiple layers. No single control should be treated as sufficient protection.

---

# 4. Threat Modeling

Use STRIDE as the initial threat-modeling framework.

STRIDE categories:

- Spoofing.
- Tampering.
- Repudiation.
- Information Disclosure.
- Denial of Service.
- Elevation of Privilege.

## 4.1 Threat Register

| ID | Threat | Category | Risk | Primary Mitigation |
|---|---|---|---|---|
| TH-001 | Stolen virtual API key | Spoofing | Critical | Hashing, rotation, revocation |
| TH-002 | Cross-tenant data access | Information Disclosure | Critical | Tenant isolation |
| TH-003 | Provider credential leakage | Information Disclosure | Critical | External secret manager |
| TH-004 | Budget manipulation | Tampering | Critical | Authorization and transactional controls |
| TH-005 | API flooding | DoS | High | Rate limiting and abuse protection |
| TH-006 | Unauthorized administration | Elevation of Privilege | Critical | RBAC and administrative authentication |
| TH-007 | Prompt data leakage through logs | Information Disclosure | High | Data minimization and redaction |
| TH-008 | SSRF through provider endpoints | Elevation of Privilege | Critical | Endpoint validation and network restrictions |
| TH-009 | Malicious configuration changes | Tampering | High | Versioning and audit logging |
| TH-010 | Replay of inference requests | Tampering | High | Idempotency and replay controls |
| TH-011 | Cache poisoning | Tampering | High | Cache integrity and tenant isolation |
| TH-012 | Malicious prompt injection | Application abuse | High | Untrusted-content boundaries |
| TH-013 | Dependency compromise | Supply-chain | High | Dependency auditing and pinning |
| TH-014 | Secret exposure in crash reports | Information Disclosure | High | Redaction and secure diagnostics |
| TH-015 | Distributed rate-limit bypass | DoS | High | Atomic shared enforcement |

Risk classifications are initial qualitative assessments. They must be revisited after deployment topology and operational controls are defined.

---

# 5. Authentication

## 5.1 Virtual API Key Authentication

Inference clients authenticate using gateway-issued API keys.

Example:

```http
Authorization: Bearer lg_live_xxxxxxxxxxxxxxxxx
```

### Key generation

Requirements:

- Use a cryptographically secure random number generator.
- Generate sufficiently high-entropy credentials.
- Include an environment or key-type prefix.
- Return the raw key only once.
- Store only a secure hash.
- Never log the raw key.

Example format:

```text
lg_live_<random-secret>
```

### Key hashing

Use a cryptographic hash designed for high-entropy API tokens.

Suitable options include:

- SHA-256 or HMAC-SHA-256 with a protected server-side pepper.
- A dedicated keyed-hash construction with controlled key rotation.

Do not use an ordinary fast password hash as the only protection for a randomly generated high-entropy API key. Password hashing algorithms are designed for low-entropy human passwords.

Store:

- Key ID.
- Key prefix.
- Hash.
- Hash version.
- Tenant ID.
- Project ID.
- Scopes.
- Expiration.
- Status.
- Creation timestamp.
- Last-used timestamp.

The prefix is for identification and must not be sufficient to authenticate.

## 5.2 Administrative Authentication

Administrative access must be separate from inference API key authentication.

Initial implementation:

- JWT authentication.
- Short-lived access tokens.
- Secure refresh-token lifecycle if refresh tokens are supported.
- Role and permission checks.
- Session revocation strategy.

Future extension:

- OIDC.
- Enterprise SSO.
- MFA enforcement.
- Identity-provider integration.

## 5.3 Authentication Failure Rules

| Condition | Response |
|---|---|
| Missing key | 401 |
| Invalid key | 401 |
| Revoked key | 401 |
| Expired key | 401 |
| Suspended tenant | 403 |
| Missing permission | 403 |

Responses must not reveal whether a particular key ID or secret exists.

---

# 6. Authorization and RBAC

Authorization must be checked for every protected operation.

## 6.1 Roles

| Role | Scope |
|---|---|
| PlatformAdmin | Entire platform |
| TenantAdmin | One tenant |
| ProjectAdmin | One project |
| Developer | Inference and permitted project resources |
| Auditor | Read-only authorized resources |

## 6.2 Permissions

Examples:

```text
inference:create
inference:read

apikey:create
apikey:read
apikey:revoke
apikey:rotate

budget:create
budget:read
budget:update

policy:create
policy:read
policy:update

model:read
model:update

provider:create
provider:read
provider:update

audit:read

tenant:manage
project:manage
```

## 6.3 Authorization Rules

- Deny by default.
- Enforce least privilege.
- Verify the requested resource belongs to the authorized tenant.
- Verify project scope.
- Do not trust tenant IDs supplied by clients.
- Separate administrative permissions from inference permissions.
- Audit sensitive authorization failures.
- Re-evaluate authorization when accessing stored resources.

An API key associated with Project A must not access Project B by changing a URL parameter.

---

# 7. Tenant Isolation

Tenant isolation is a critical security boundary.

## 7.1 Tenant Scoping

Every tenant-owned resource must have an explicit ownership relationship.

Examples:

- Projects.
- API keys.
- Policies.
- Budgets.
- Usage records.
- Inference executions.
- Cache entries.
- Evaluation data.
- Audit records.

## 7.2 Database Isolation

Every query must be scoped by tenant ownership.

Example:

```sql
SELECT *
FROM inference_executions
WHERE id = $1
  AND tenant_id = $2;
```

Never retrieve a tenant-owned resource by ID alone when the operation is authorized in a tenant context.

Additional defense:

- PostgreSQL Row-Level Security where appropriate.
- Explicit repository-level tenant filters.
- Integration tests attempting cross-tenant access.
- Tenant-aware cache keys.
- Tenant-aware background jobs.

Database Row-Level Security is defense in depth, not a replacement for application authorization.

## 7.3 Cache Isolation

Cache keys must include tenant and project context.

Example:

```text
cache:{tenant_id}:{project_id}:{request_fingerprint}
```

Semantic cache lookup must also enforce:

- Authorization.
- Tenant ownership.
- Model compatibility.
- Prompt configuration compatibility.
- Data classification.
- Cache expiration.

A similarity score alone must never authorize returning a cached response.

---

# 8. Secret Management

## 8.1 Secret Categories

Two credential categories must remain separate.

### Gateway credentials

- Virtual API keys.
- Administrative credentials.
- JWT signing keys.
- Internal service credentials.

### Provider credentials

- OpenAI credentials.
- Anthropic credentials.
- Other provider credentials.
- Internal inference endpoint credentials.

## 8.2 Secret Storage

Development:

- Environment variables.
- Docker secrets.
- Local development secret files excluded from Git.

Production:

- HashiCorp Vault.
- AWS Secrets Manager.
- Azure Key Vault.
- Google Secret Manager.

The system should implement a `SecretProvider` abstraction.

```rust
#[async_trait::async_trait]
pub trait SecretProvider: Send + Sync {
    async fn get_secret(
        &self,
        secret_name: &str,
    ) -> Result<SecretValue, SecretError>;
}
```

## 8.3 Secret Rotation

Requirements:

1. Support credential replacement.
2. Support secret versioning where available.
3. Avoid application restarts when safe rotation is possible.
4. Avoid logging secret values.
5. Record rotation events.
6. Support disabling compromised credentials.
7. Avoid returning provider credentials through administrative APIs.

Rotation must account for provider behavior and any overlap period during credential replacement.

## 8.4 Secret Access Control

- Only provider adapter infrastructure should receive provider credentials.
- Domain entities store references, not secret values.
- Administrative users manage references, not retrieve plaintext credentials.
- Secret retrieval must be audited without recording values.
- Access should use the minimum required permissions.

---

# 9. Network Security and SSRF Protection

Provider endpoints may be configurable. This creates an SSRF risk.

## 9.1 SSRF Threat

An attacker could attempt to configure a provider endpoint that points to:

- Cloud metadata services.
- Internal management APIs.
- Private infrastructure.
- Localhost services.
- Internal database endpoints.

## 9.2 Mitigations

- Prefer approved provider endpoint allowlists.
- Validate URL schemes.
- Require HTTPS for external providers.
- Reject loopback and link-local destinations where not explicitly permitted.
- Block cloud metadata endpoints.
- Validate DNS resolution.
- Revalidate destinations after DNS changes.
- Restrict outbound network traffic.
- Disable redirects by default for provider requests.
- Revalidate redirect destinations if redirects are explicitly supported.

DNS rebinding must be considered. URL validation alone is insufficient if the actual resolved destination is not controlled.

For self-hosted providers, use explicit administrative configuration and network allowlists.

---

# 10. Request Validation and Abuse Protection

## 10.1 Request Validation

Validate:

- HTTP method.
- Content type.
- Request body size.
- JSON structure.
- Message count.
- Message role.
- Input token estimates.
- Maximum output tokens.
- Sampling parameters.
- Model capabilities.
- Tool definitions.
- Metadata size.
- Supported content modalities.

Reject invalid requests before expensive provider operations.

## 10.2 Abuse Protection

Detect:

- Excessive authentication failures.
- High request bursts.
- Oversized payloads.
- Repeated malformed requests.
- Excessive concurrency.
- Suspicious API key usage patterns.
- Unusual spending patterns.

Actions may include:

- Temporary throttling.
- API key suspension.
- Tenant-level throttling.
- Administrative alerting.
- Security audit events.

Automated blocking must use documented thresholds and avoid accidentally disabling legitimate high-volume tenants.

---

# 11. Rate Limiting and Resource Protection

Rate limiting and concurrency controls must be enforced across gateway instances.

## 11.1 Rate Limit Dimensions

- Requests per second.
- Requests per minute.
- Input tokens per minute.
- Output tokens per minute.
- Tenant quota.
- Project quota.
- API key quota.
- Provider quota.

## 11.2 Concurrency Controls

Enforce maximum simultaneous requests at:

- Global level.
- Tenant level.
- Project level.
- Provider level.

## 11.3 Redis Security

- Require authentication.
- Use TLS where supported and required.
- Restrict network access.
- Use least-privilege credentials.
- Avoid exposing Redis to public networks.
- Use appropriate key expiration.
- Protect atomic scripts and operations.
- Monitor memory consumption.

Define fail-open or fail-closed behavior independently for each control.

For example, budget admission and tenant isolation must not be bypassed simply because Redis is unavailable.

---

# 12. Financial Security

Budget enforcement is a security control.

Requirements:

- Atomic budget reservations.
- Idempotent settlement.
- Transactional usage accounting.
- No floating-point financial calculations.
- Authorized budget modifications.
- Audit all budget changes.
- Prevent negative balances.
- Prevent concurrent reservation races.
- Define provider timeout reconciliation.
- Monitor unusual expenditure.

Financial invariants must be enforced transactionally in PostgreSQL.

Redis must not be the sole authority for financial balances.

---

# 13. Prompt Injection and Untrusted Content

Prompt injection is primarily an application-level threat involving untrusted content influencing model behavior.

The gateway should distinguish between:

1. Gateway control instructions.
2. Client-supplied messages.
3. Retrieved or external content.
4. Tool definitions and execution permissions.

## 13.1 Gateway Responsibilities

- Preserve message roles and boundaries.
- Avoid interpreting user content as gateway configuration.
- Enforce model and provider policies independently of prompt content.
- Validate tool-call structures.
- Prevent client-supplied metadata from overriding trusted routing configuration.
- Apply configured content safety policies.
- Record security-relevant policy outcomes.

## 13.2 Tool Calling

The gateway should not automatically execute arbitrary tools simply because an LLM requests them.

Tool execution requires:

- Explicit tool registration.
- Authorization.
- Schema validation.
- Policy enforcement.
- Resource limits.
- Audit logging.

The initial gateway should treat tool execution as an explicitly controlled extension rather than an implicit provider feature.

---

# 14. PII and Data Protection

## 14.1 Data Classification

Initial classifications:

| Classification | Examples | Handling |
|---|---|---|
| Public | Public documentation | Normal processing |
| Internal | Non-public business content | Restricted access |
| Confidential | Customer business data | Strict access and retention |
| Sensitive | Personal or regulated information | Explicit handling policy |

The classification should be configurable and aligned with the deployment's legal and contractual requirements.

## 14.2 PII Protection

Support:

- Configurable PII detection.
- Redaction policies.
- Restricted prompt retention.
- Log redaction.
- Evaluation data minimization.
- Tenant-specific retention rules.

PII detection is not guaranteed to identify every sensitive value. It should be treated as a defense-in-depth control.

## 14.3 Data Retention

Retention must be defined independently for:

- Prompts and responses.
- Inference metadata.
- Usage records.
- Audit events.
- Traces and logs.
- Evaluation datasets.
- Evaluation results.

Do not retain full prompts and responses by default unless the customer policy and business purpose justify it.

---

# 15. Encryption

## 15.1 Encryption in Transit

- TLS for public APIs.
- TLS for external provider communication.
- TLS for internal connections where required by deployment risk.
- Secure certificate validation.
- No disabled certificate verification in production.

## 15.2 Encryption at Rest

Encrypt:

- Database storage.
- Backups.
- Sensitive configuration.
- Secret-manager data.
- Persistent observability data where appropriate.

Use managed encryption keys or an approved KMS.

Encryption key access must be separated from ordinary application database access.

## 15.3 Key Management

- Define key ownership.
- Define key rotation.
- Restrict decryption permissions.
- Audit key-management operations.
- Protect backup encryption keys.
- Document recovery procedures.

---

# 16. Audit Logging

Audit logging is required for security-sensitive actions.

Record:

- API key creation.
- API key revocation.
- API key rotation.
- Authentication failures.
- Authorization failures.
- Tenant suspension.
- Policy modifications.
- Budget modifications.
- Provider configuration changes.
- Secret reference changes.
- Administrative actions.
- Security-related blocks.

Audit event example:

```json
{
  "event_type": "API_KEY_REVOKED",
  "actor_id": "user_123",
  "tenant_id": "tenant_123",
  "resource_type": "api_key",
  "resource_id": "key_123",
  "outcome": "success",
  "occurred_at": "2026-11-03T12:00:00Z"
}
```

Never include:

- Raw API keys.
- Provider credentials.
- JWT signing keys.
- Passwords.
- Unnecessary prompt or response content.

Audit records must be access-controlled and protected against unauthorized modification.

---

# 17. Observability Security

Traces, metrics, and logs can expose sensitive information if incorrectly configured.

Requirements:

- Structured logs.
- Redaction before export.
- No authorization headers in logs.
- No provider secrets in spans.
- No raw API key values in attributes.
- Restricted access to trace backends.
- Configurable prompt and response capture.
- Tenant-aware observability access.
- Defined telemetry retention.

Use safe identifiers such as:

- Request ID.
- Execution ID.
- Tenant ID where permitted.
- Provider ID.
- Model ID.
- Policy version.

Sensitive content should be opt-in for explicitly authorized diagnostic workflows, not the default.

---

# 18. Idempotency and Replay Protection

The gateway must protect against accidental and malicious duplicate requests.

Requirements:

- Scope idempotency keys by tenant and project.
- Bind keys to request fingerprints.
- Reject reuse with different payloads.
- Handle concurrent requests using the same key.
- Apply expiration and retention rules.
- Audit administrative idempotency overrides.

The gateway must distinguish client retries from provider retries.

An external provider may process a request even when the gateway does not receive its response. The gateway must represent uncertain outcomes rather than assuming that a timeout means no provider execution occurred.

---

# 19. Dependency and Supply-Chain Security

The Rust workspace must enforce dependency hygiene.

Requirements:

- Commit `Cargo.lock`.
- Pin and review dependency updates.
- Run `cargo audit`.
- Run `cargo deny`.
- Review unsafe Rust usage.
- Run static analysis.
- Generate an SBOM.
- Scan container images.
- Use minimal runtime images.
- Avoid unnecessary OS packages.
- Protect CI secrets.
- Use trusted build actions.
- Require review for sensitive dependency changes.

Recommended checks:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo audit
cargo deny check
```

These checks should run in CI.

A security scan passing does not prove that the system is secure. Findings must be triaged and documented.

---

# 20. Security Testing Strategy

Security testing must cover domain, application, infrastructure, and API boundaries.

## 20.1 Unit Tests

- API key hash verification.
- Permission evaluation.
- Tenant ownership checks.
- Budget invariants.
- Rate-limit decision logic.
- Secret reference validation.
- Policy evaluation.
- Request validation.

## 20.2 Integration Tests

- Invalid API key rejection.
- Revoked key rejection.
- Expired key rejection.
- Cross-tenant resource access.
- Cross-project resource access.
- Budget reservation races.
- Duplicate idempotency requests.
- Provider credential resolution.
- Secret manager failure.
- Redis failure behavior.
- PostgreSQL failure behavior.

## 20.3 Security Tests

- SQL injection.
- SSRF.
- Authorization bypass.
- Mass assignment.
- Cache poisoning.
- Request smuggling where relevant to the deployment.
- Oversized payloads.
- Rate-limit bypass.
- Secret leakage.
- Prompt injection boundary tests.
- Dependency vulnerability checks.

## 20.4 Load and Resilience Tests

- High request rates.
- Concurrent budget reservations.
- Redis interruption.
- Database connection exhaustion.
- Provider timeouts.
- Provider outages.
- Slow streaming consumers.
- Client disconnects.
- Recovery after dependency failure.

---

# 21. Incident Response

The initial incident response process must define:

1. Detection.
2. Triage.
3. Containment.
4. Eradication.
5. Recovery.
6. Post-incident review.

Examples of security incidents:

- Compromised virtual API key.
- Leaked provider credential.
- Cross-tenant data exposure.
- Unauthorized budget modification.
- Malicious provider endpoint configuration.
- Suspicious spending surge.
- Abuse of inference endpoints.

Containment actions should include:

- Revoke compromised keys.
- Rotate affected provider credentials.
- Suspend affected projects or tenants.
- Block malicious traffic.
- Disable affected provider configurations.
- Preserve relevant audit evidence.
- Notify authorized stakeholders according to incident policy.

The system must provide operational procedures for credential rotation and recovery.

---

# 22. Security Configuration

Security-related settings must be externalized and validated.

Example:

```yaml
security:
  api_keys:
    minimum_entropy_bits: 256
    show_once: true
    hash_algorithm: hmac_sha256

  authentication:
    allow_expired_keys: false
    allow_revoked_keys: false

  request:
    max_body_size_bytes: 1048576
    max_messages: 100
    require_content_type: true

  tenant_isolation:
    enforce_tenant_scope: true
    enforce_project_scope: true

  secrets:
    provider: environment
    cache_ttl_seconds: 300
    allow_plaintext_logging: false

  pii:
    enabled: true
    redact_logs: true
    retain_prompts: false

  network:
    require_https_for_external_providers: true
    allow_redirects: false

  audit:
    enabled: true
    record_authentication_failures: true
    record_authorization_failures: true
```

This is an illustrative configuration contract. Actual field names and supported settings must be validated by the implementation.

Production must not use insecure defaults simply because a setting is omitted.

---

# 23. Security Acceptance Criteria

The security architecture is ready for implementation when:

- [ ] Threat register is documented.
- [ ] Authentication flows are defined.
- [ ] API key generation and hashing are defined.
- [ ] API key rotation and revocation are defined.
- [ ] Administrative authentication is separated.
- [ ] RBAC and permission scopes are defined.
- [ ] Tenant isolation strategy is documented.
- [ ] Secret manager abstraction is defined.
- [ ] Secret rotation procedure is defined.
- [ ] SSRF protections are specified.
- [ ] Request validation is defined.
- [ ] Rate limiting and concurrency protection are defined.
- [ ] Financial controls are documented.
- [ ] Prompt injection boundaries are documented.
- [ ] PII and retention policies are defined.
- [ ] Encryption requirements are defined.
- [ ] Audit logging requirements are defined.
- [ ] Observability redaction is defined.
- [ ] Idempotency and replay controls are defined.
- [ ] Dependency security checks are defined.
- [ ] Security test strategy is documented.
- [ ] Incident response process is documented.

---

# 24. Next Document

The next specification is `docs/routing-policy.md`.

It will define:

- Model eligibility.
- Model capability matching.
- Task classification.
- Complexity estimation.
- Quality constraints.
- Cost optimization.
- Latency constraints.
- Provider health.
- Data residency.
- Routing rules and priorities.
- Fallback policies.
- Retry policies.
- Routing explainability.
- Routing evaluation and benchmarking.

The core routing objective remains:

**Select the lowest-cost eligible model that satisfies the request's capability, quality, security, reliability, and latency constraints.**
