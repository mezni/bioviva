
# ADR-0023: API Versioning and OpenAPI Governance

- **Status:** Proposed
- **Date:** 2026-11-04
- **Decision Owners:** llm-gateway architecture team
- **Related ADRs:** ADR-0001, ADR-0006, ADR-0007, ADR-0010, ADR-0014, ADR-0019, ADR-0020

## 1. Context

The `llm-gateway` exposes APIs to client applications, administrative users, and internal services.

These APIs are consumed by independently deployed applications and must evolve without introducing unexpected breaking changes.

The gateway needs a consistent API governance strategy covering:

- Versioning
- Request and response contracts
- Error formats
- Authentication and authorization
- Streaming
- Pagination
- Idempotency
- Deprecation
- OpenAPI documentation
- Contract testing
- SDK generation

Without a defined API governance model, changes to provider integrations, routing policies, and inference behavior could unintentionally break existing clients.

## 2. Decision

The gateway will adopt:

1. URL-based major API versioning.
2. OpenAPI 3.1 as the API contract specification.
3. Provider-neutral public request and response models.
4. Backward-compatible changes within a major version.
5. Explicit deprecation and retirement policies.
6. Automated OpenAPI validation and contract testing in CI.
7. Separate public inference APIs, administrative APIs, and internal service APIs.

The initial public API version will be `v1`.

## 3. API Boundaries

The gateway exposes three API boundaries.

| API | Consumers | Purpose |
|---|---|---|
| Inference API | Client applications | Model inference, streaming, usage |
| Administrative API | Platform and tenant administrators | Configuration and management |
| Internal API | Trusted internal services | Worker communication and internal operations |

Each API has its own authentication and authorization requirements.

### 3.1 API structure

```text
/api/
├── v1/
│   ├── chat/completions
│   ├── responses
│   ├── models
│   ├── executions
│   └── usage
│
/admin/
└── v1/
    ├── tenants
    ├── projects
    ├── api-keys
    ├── providers
    ├── models
    ├── routing-policies
    ├── budgets
    └── audit-events

/internal/
└── v1/
    ├── evaluation-jobs
    ├── reconciliation
    └── health
```

Internal endpoints must not be exposed through the public API gateway.

## 4. Versioning Strategy

### 4.1 URL-based major versions

Public APIs use:

```text
/api/v1/...
/api/v2/...
```

Administrative APIs use:

```text
/admin/v1/...
/admin/v2/...
```

A new major version is required for breaking changes.

### 4.2 Non-breaking changes

The following changes are generally allowed within an existing major version:

- Adding optional request fields.
- Adding response fields.
- Adding new endpoints.
- Adding new documented error codes.
- Adding optional query parameters.
- Introducing new model identifiers.

Clients must ignore unknown response fields.

The gateway must not assume that all clients will immediately adopt newly introduced capabilities.

### 4.3 Breaking changes

Examples:

- Removing a response field.
- Changing a field's meaning.
- Changing a field's type.
- Making an optional field mandatory.
- Changing authentication semantics.
- Changing error response structure incompatibly.
- Changing streaming event semantics.
- Changing idempotency behavior incompatibly.

Breaking changes require a new major version.

## 5. Public Inference API

The public inference API must use provider-neutral contracts.

Clients interact with gateway model identifiers rather than provider-specific model credentials or endpoints.

Example:

```http
POST /api/v1/chat/completions
Authorization: Bearer lgw_live_...
Content-Type: application/json
```

Request:

```json
{
  "model": "gateway-fast",
  "messages": [
    {
      "role": "user",
      "content": "Explain distributed systems."
    }
  ],
  "temperature": 0.2,
  "max_tokens": 500,
  "stream": false
}
```

Response:

```json
{
  "id": "exec_123",
  "object": "chat.completion",
  "model": "gateway-fast",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Distributed systems..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 100,
    "total_tokens": 120
  }
}
```

The public response must not expose internal provider credentials or sensitive routing information.

The gateway may expose selected provider metadata only through explicitly authorized diagnostic interfaces.

## 6. API Error Contract

All public API errors must follow a consistent structure.

Example:

```json
{
  "error": {
    "type": "authentication_error",
    "code": "invalid_api_key",
    "message": "Authentication failed.",
    "request_id": "req_123"
  }
}
```

### 6.1 Error categories

| HTTP status | Category | Example |
|---|---|---|
| 400 | Invalid request | `invalid_request` |
| 401 | Authentication | `invalid_api_key` |
| 403 | Authorization | `permission_denied` |
| 404 | Resource | `resource_not_found` |
| 409 | Conflict | `idempotency_conflict` |
| 413 | Payload | `request_too_large` |
| 422 | Policy | `budget_exceeded` |
| 429 | Rate limit | `rate_limit_exceeded` |
| 502 | Provider failure | `provider_error` |
| 503 | Availability | `service_unavailable` |
| 504 | Timeout | `provider_timeout` |

Error responses must not expose:

- API keys
- Provider secrets
- Internal stack traces
- Database connection details
- Secret-manager references
- Sensitive tenant information

Error codes must remain stable within a major API version.

## 7. Streaming Contract

Streaming inference uses Server-Sent Events (SSE).

Example:

```http
Content-Type: text/event-stream
Cache-Control: no-cache
```

Example events:

```text
event: response.created
data: {"id":"exec_123","status":"in_progress"}

event: response.output_text.delta
data: {"delta":"Hello"}

event: response.output_text.delta
data: {"delta":" world"}

event: response.completed
data: {"id":"exec_123","status":"completed"}
```

The contract must define:

- Event names.
- Event payload schemas.
- Event ordering.
- Terminal events.
- Error behavior.
- Client cancellation behavior.
- Usage reporting.
- Connection timeout behavior.

Streaming semantics must remain consistent with ADR-0014.

Once output becomes client-visible, transparent provider fallback is prohibited unless the API contract explicitly defines a safe recovery mechanism.

## 8. Idempotency Contract

Inference endpoints that support idempotency must accept:

```http
Idempotency-Key: <client-generated-key>
```

The gateway must define:

- Key scope.
- Retention period.
- Request fingerprinting.
- Duplicate request behavior.
- Conflict behavior.
- Execution recovery behavior.

A repeated request with the same idempotency key and equivalent request payload must not accidentally trigger duplicate provider execution.

Reusing an idempotency key with a different request payload must return a conflict.

Detailed lifecycle behavior is defined in ADR-0010.

## 9. Pagination and Filtering

Administrative list endpoints must use cursor-based pagination.

Example:

```http
GET /admin/v1/tenants?limit=50&cursor=abc123
```

Response:

```json
{
  "data": [],
  "pagination": {
    "next_cursor": "def456",
    "has_more": true
  }
}
```

Requirements:

- Enforce a maximum page size.
- Use stable ordering.
- Avoid offset pagination for large operational datasets.
- Validate filtering and sorting fields.
- Enforce authorization before returning results.
- Ensure cursors do not grant access beyond the caller's authorized scope.

## 10. OpenAPI Governance

OpenAPI 3.1 will be the authoritative API contract.

Specifications will be stored in version control.

Suggested structure:

```text
openapi/
├── public/
│   └── v1.yaml
├── admin/
│   └── v1.yaml
├── internal/
│   └── v1.yaml
├── components/
│   ├── schemas/
│   ├── parameters/
│   ├── responses/
│   └── security-schemes/
└── README.md
```

### 10.1 Contract ownership

Every endpoint must have:

- Defined owner.
- Request schema.
- Response schema.
- Authentication requirement.
- Authorization requirement.
- Error responses.
- Operational behavior.
- Examples.
- Compatibility expectations.

Undocumented endpoints must not be exposed as supported public APIs.

### 10.2 CI validation

CI must validate:

1. OpenAPI syntax.
2. Schema consistency.
3. Duplicate operation IDs.
4. Missing security definitions.
5. Missing error responses.
6. Breaking changes.
7. Request and response examples.
8. Generated SDK compatibility where applicable.

A breaking change within an existing major version must fail CI unless explicitly approved through the versioning process.

## 11. API Compatibility Policy

Use the following compatibility rules:

| Change | Compatibility |
|---|---|
| Add optional field | Non-breaking |
| Add endpoint | Non-breaking |
| Add optional query parameter | Non-breaking |
| Remove field | Breaking |
| Rename field | Breaking |
| Change field type | Breaking |
| Change error semantics | Potentially breaking |
| Change authentication requirements | Breaking |
| Change SSE event semantics | Breaking |

Be cautious with adding enum values: generated SDKs may model enums as closed sets. Where practical, document unknown-value handling or use extensible string fields.

## 12. Deprecation and Retirement

The gateway must have a documented deprecation process.

Recommended process:

1. Announce deprecation.
2. Document replacement behavior.
3. Publish migration guidance.
4. Add deprecation metadata and response headers where appropriate.
5. Monitor remaining usage.
6. Notify affected customers.
7. Retire only after the published support period and contractual obligations are satisfied.

Suggested headers:

```http
Deprecation: true
Sunset: Wed, 30 Jun 2027 00:00:00 GMT
Link: </docs/migrations/v1-to-v2>; rel="deprecation"
```

Deprecation dates must be explicit and communicated.

The initial support window for a major version should be defined in the product's release policy rather than hard-coded into the API implementation.

## 13. Rust Implementation

### API contract types

Public request and response types belong in a dedicated contract module.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
}
```

Avoid exposing internal domain entities directly through HTTP responses.

Map between API DTOs and application/domain models explicitly.

### Suggested structure

```text
crates/
├── gateway-api/
│   ├── routes/
│   │   ├── public_v1.rs
│   │   ├── admin_v1.rs
│   │   └── internal_v1.rs
│   ├── contracts/
│   │   ├── inference.rs
│   │   ├── execution.rs
│   │   └── errors.rs
│   └── middleware/
└── gateway-core/
    └── inference/
        └── models.rs

openapi/
├── public/v1.yaml
├── admin/v1.yaml
└── internal/v1.yaml
```

## 14. SDK and Client Strategy

The OpenAPI contracts should support generated clients.

Initial SDK candidates:

- Python
- TypeScript

SDKs must:

- Use gateway virtual API keys.
- Avoid embedding credentials in distributed frontend code.
- Support request IDs.
- Support idempotency where available.
- Support streaming.
- Expose typed error categories.
- Preserve unknown response fields where practical.

SDKs must be versioned independently from the API contract.

## 15. Observability and Documentation

Every request should have:

- Request ID
- API version
- Operation identifier
- HTTP status
- Latency
- Authenticated tenant/project context
- Execution ID where applicable

Do not include secrets or sensitive prompt content in standard request logs.

API documentation must include:

- Authentication instructions.
- Request and response examples.
- Error reference.
- Rate-limit behavior.
- Idempotency semantics.
- Streaming examples.
- Versioning policy.
- Migration guides.

## 16. Testing Strategy

### Contract tests

- Request schema validation.
- Response schema validation.
- Error schema validation.
- Required security definitions.
- Streaming event schema validation.
- Pagination behavior.

### Compatibility tests

- OpenAPI breaking-change detection.
- Existing client compatibility.
- Unknown response-field handling.
- Enum extension behavior.
- Version routing behavior.

### Integration tests

- Authentication per API boundary.
- Authorization enforcement.
- Idempotency behavior.
- Cursor pagination.
- SSE event ordering.
- Deprecation headers.
- Request ID propagation.

### Security tests

- Internal endpoint exposure.
- Administrative endpoint authorization.
- Sensitive information in error responses.
- Credential leakage through generated documentation.
- Cross-tenant resource access.

## 17. Consequences

### Positive

- Stable client integration contracts.
- Clear separation of public, administrative, and internal APIs.
- Automated compatibility checks.
- Consistent error and streaming semantics.
- Easier SDK generation and client onboarding.

### Negative

- Additional contract maintenance.
- API DTO mapping overhead.
- Multiple versions may need to be maintained simultaneously.
- OpenAPI governance adds CI requirements.

### Risks

- Implementation may drift from OpenAPI contracts.
- Poor deprecation management can break customers.
- Generated SDKs may mishandle extensible response fields.
- Streaming behavior may change without sufficient contract coverage.

## 18. Validation Criteria

- [ ] Public APIs use URL-based major versions.
- [ ] Inference, administrative, and internal APIs are separated.
- [ ] OpenAPI 3.1 specifications are version-controlled.
- [ ] Every public endpoint has a documented contract.
- [ ] API DTOs are separate from internal domain models.
- [ ] Error responses follow a stable structure.
- [ ] Streaming event semantics are documented.
- [ ] Idempotency behavior is documented.
- [ ] Administrative pagination uses stable cursors.
- [ ] Breaking changes fail CI without approval.
- [ ] Deprecation and retirement policies are documented.
- [ ] Generated SDKs pass compatibility tests.
- [ ] Sensitive data is excluded from API errors and documentation.

## 19. Related Decisions

- ADR-0001: Clean Architecture and Dependency Direction
- ADR-0006: Canonical Provider Adapter Interface
- ADR-0007: Model Registry and Capability Representation
- ADR-0010: Inference Idempotency and Execution Lifecycle
- ADR-0014: Streaming and Provider Fallback Semantics
- ADR-0019: Virtual API Key Security Model
- ADR-0020: Administrative Authentication and Authorization

