# LLM Gateway — API Contracts

**Version:** 1.0  
**Status:** Draft  
**Project:** `llm-gateway`  
**API Style:** REST  
**Transport:** HTTPS  
**Payload:** JSON  
**Streaming:** Server-Sent Events (SSE)  
**Implementation:** Rust + Axum

---

## 1. Purpose

This document defines the external API contract exposed by the LLM Gateway.

It establishes:

- API conventions and versioning.
- Authentication and authorization.
- Inference endpoints.
- Streaming inference.
- Model discovery.
- API key administration.
- Tenant and project management.
- Policy management.
- Budget management.
- Rate-limit and concurrency behavior.
- Usage reporting.
- Error handling.
- Idempotency.
- Request and response schemas.

The gateway exposes a stable interface independent of the underlying LLM provider.

---

## 2. API Design Principles

1. Provider-independent request and response formats.
2. Explicit API versioning.
3. Consistent error responses.
4. Strict request validation.
5. Tenant and project isolation.
6. Explicit idempotency support.
7. Secure credential handling.
8. Consistent pagination.
9. Streaming support.
10. Backward-compatible contract evolution.

### Base URL

Development:

```text
http://localhost:8080
```

Production example:

```text
https://api.example.com
```

API version prefix:

```text
/v1
```

---

## 3. API Authentication

### 3.1 Virtual API Key

Inference clients authenticate using a gateway-issued API key.

Header:

```http
Authorization: Bearer lg_live_xxxxxxxxxxxxxxxxx
```

Example:

```http
POST /v1/chat/completions HTTP/1.1
Host: localhost:8080
Authorization: Bearer lg_live_xxxxxxxxxxxxxxxxx
Content-Type: application/json
```

The gateway must:

- Hash and verify the key.
- Check expiration and revocation.
- Resolve tenant and project ownership.
- Resolve permitted scopes.
- Apply tenant and project policies.
- Update last-used metadata asynchronously where appropriate.

The raw API key is returned only once during creation.

### 3.2 Administrative Authentication

Administrative APIs use a separate authentication mechanism.

Initial implementation options:

- JWT-based authentication.
- OIDC integration in a later stage.

Administrative authorization must enforce RBAC and resource scope.

Do not use inference API keys as administrative credentials.

---

## 4. API Endpoint Overview

### 4.1 Inference

| Method | Endpoint | Description |
|---|---|---|
| POST | `/v1/chat/completions` | Chat inference |
| POST | `/v1/chat/completions` with `stream=true` | Streaming chat inference |
| POST | `/v1/embeddings` | Generate embeddings |
| GET | `/v1/models` | List available models |
| GET | `/v1/models/{model_id}` | Retrieve model details |

### 4.2 API Key Management

| Method | Endpoint | Description |
|---|---|---|
| POST | `/v1/admin/api-keys` | Create API key |
| GET | `/v1/admin/api-keys` | List API keys |
| GET | `/v1/admin/api-keys/{id}` | Retrieve API key metadata |
| POST | `/v1/admin/api-keys/{id}/revoke` | Revoke API key |
| POST | `/v1/admin/api-keys/{id}/rotate` | Rotate API key |

### 4.3 Tenant and Project Management

| Method | Endpoint | Description |
|---|---|---|
| GET | `/v1/admin/tenants` | List tenants |
| POST | `/v1/admin/tenants` | Create tenant |
| GET | `/v1/admin/tenants/{id}` | Retrieve tenant |
| PATCH | `/v1/admin/tenants/{id}` | Update tenant |
| POST | `/v1/admin/tenants/{id}/suspend` | Suspend tenant |
| GET | `/v1/admin/projects` | List authorized projects |
| POST | `/v1/admin/projects` | Create project |
| GET | `/v1/admin/projects/{id}` | Retrieve project |
| PATCH | `/v1/admin/projects/{id}` | Update project |

### 4.4 Policy Management

| Method | Endpoint | Description |
|---|---|---|
| GET | `/v1/admin/policies` | List policies |
| POST | `/v1/admin/policies` | Create policy |
| GET | `/v1/admin/policies/{id}` | Retrieve policy |
| PUT | `/v1/admin/policies/{id}` | Create a new policy version |
| POST | `/v1/admin/policies/{id}/activate` | Activate policy |
| POST | `/v1/admin/policies/{id}/disable` | Disable policy |

### 4.5 Budget and Usage

| Method | Endpoint | Description |
|---|---|---|
| GET | `/v1/admin/budgets` | List budgets |
| POST | `/v1/admin/budgets` | Create budget |
| GET | `/v1/admin/budgets/{id}` | Retrieve budget |
| PATCH | `/v1/admin/budgets/{id}` | Update budget |
| GET | `/v1/usage` | Retrieve authorized usage |
| GET | `/v1/usage/{execution_id}` | Retrieve execution usage |

### 4.6 Provider and Model Administration

| Method | Endpoint | Description |
|---|---|---|
| GET | `/v1/admin/providers` | List providers |
| POST | `/v1/admin/providers` | Register provider |
| PATCH | `/v1/admin/providers/{id}` | Update provider |
| GET | `/v1/admin/providers/{id}/health` | Retrieve provider health |
| POST | `/v1/admin/providers/{id}/test` | Test provider connection |
| POST | `/v1/admin/models` | Register model |
| PATCH | `/v1/admin/models/{id}` | Update model |

### 4.7 Operational Endpoints

| Method | Endpoint | Description |
|---|---|---|
| GET | `/health/live` | Liveness |
| GET | `/health/ready` | Readiness |
| GET | `/metrics` | Prometheus metrics, restricted to monitoring access |

---

# 5. Chat Completion API

## 5.1 Request

`POST /v1/chat/completions`

Example:

```json
{
  "model": "auto",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant."
    },
    {
      "role": "user",
      "content": "Explain Rust ownership."
    }
  ],
  "temperature": 0.2,
  "top_p": 1.0,
  "max_tokens": 1000,
  "stream": false,
  "metadata": {
    "application": "support-assistant",
    "environment": "development"
  }
}
```

### Field definitions

| Field | Type | Required | Description |
|---|---|---|---|
| model | String | Yes | Model identifier or `auto` |
| messages | Array | Yes | Conversation messages |
| temperature | Number | No | Sampling temperature |
| top_p | Number | No | Nucleus sampling |
| max_tokens | Integer | No | Maximum output tokens |
| stream | Boolean | No | Enable streaming |
| metadata | Object | No | Client-defined metadata |

The exact supported parameters depend on the normalized gateway contract and model capabilities.

Unsupported parameters must produce a validation error rather than being silently ignored.

The domain and API contract should define a consistent output-token parameter. Provider adapters translate that parameter into provider-specific fields such as `max_tokens` or `max_completion_tokens` where necessary.

## 5.2 Message Structure

```json
{
  "role": "user",
  "content": "Hello"
}
```

Supported roles:

- system
- developer
- user
- assistant
- tool

Tool calling and multimodal message formats should be added through explicitly versioned schemas.

## 5.3 Successful Response

HTTP 200:

```json
{
  "id": "exec_01JXYZ123",
  "object": "chat.completion",
  "created": 1790000000,
  "model": "provider/model-name",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Rust ownership is a memory management model..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "input_tokens": 120,
    "output_tokens": 85,
    "total_tokens": 205
  },
  "gateway": {
    "request_id": "req_01JXYZ",
    "execution_id": "exec_01JXYZ123",
    "provider": "example-provider",
    "routing_reason": "lowest_cost_eligible_model",
    "estimated_cost": {
      "amount": "0.000420",
      "currency": "USD"
    },
    "actual_cost": {
      "amount": "0.000420",
      "currency": "USD"
    },
    "cache_hit": false
  }
}
```

The response must distinguish estimated and actual cost. Actual cost may be unavailable immediately if provider billing reconciliation is asynchronous.

The gateway metadata is an extension to the normalized completion contract.

---

# 6. Streaming API

Endpoint:

`POST /v1/chat/completions`

Request:

```json
{
  "model": "auto",
  "messages": [
    {
      "role": "user",
      "content": "Explain distributed systems."
    }
  ],
  "stream": true
}
```

Response headers:

```http
Content-Type: text/event-stream
Cache-Control: no-cache
Connection: keep-alive
```

Example stream:

```text
data: {"id":"exec_123","choices":[{"delta":{"role":"assistant"}}]}

data: {"id":"exec_123","choices":[{"delta":{"content":"Distributed"}}]}

data: {"id":"exec_123","choices":[{"delta":{"content":" systems"}}]}

data: {"id":"exec_123","choices":[{"delta":{"content":" coordinate..."}}]}

data: {"id":"exec_123","choices":[{"finish_reason":"stop"}]}

data: [DONE]
```

### Streaming requirements

- Support cancellation when the client disconnects.
- Propagate cancellation to the provider where supported.
- Enforce idle and total execution timeouts.
- Track usage when available.
- Release concurrency permits.
- Record partial executions.
- Avoid provider switching after response content has been emitted.
- Emit terminal errors using the documented SSE error format.

A connection failure after partial output must not be represented as a normal completed response.

---

# 7. Embeddings API

Endpoint:

`POST /v1/embeddings`

Request:

```json
{
  "model": "auto",
  "input": [
    "LLM Gateway architecture",
    "Model routing and cost optimization"
  ]
}
```

Response:

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "index": 0,
      "embedding": [0.012, -0.031, 0.084]
    },
    {
      "object": "embedding",
      "index": 1,
      "embedding": [0.022, -0.041, 0.064]
    }
  ],
  "model": "provider/embedding-model",
  "usage": {
    "input_tokens": 25,
    "total_tokens": 25
  },
  "gateway": {
    "request_id": "req_embedding_123",
    "execution_id": "exec_embedding_123",
    "provider": "example-provider"
  }
}
```

Embedding dimensions must match the selected model. The gateway must not silently change embedding dimensions through fallback.

---

# 8. Model Discovery API

## 8.1 List Models

`GET /v1/models`

Example response:

```json
{
  "object": "list",
  "data": [
    {
      "id": "auto",
      "object": "model",
      "capabilities": [
        "text_generation",
        "streaming"
      ]
    },
    {
      "id": "provider/model-a",
      "object": "model",
      "provider": "provider",
      "context_window": 128000,
      "max_output_tokens": 8192,
      "capabilities": [
        "text_generation",
        "streaming",
        "tool_calling"
      ]
    }
  ]
}
```

Only models permitted by the authenticated tenant's policy should be exposed.

## 8.2 Retrieve Model

`GET /v1/models/{model_id}`

Returns:

- Model identifier.
- Provider.
- Capabilities.
- Context window.
- Maximum output tokens.
- Availability.
- Pricing metadata where authorized.

Provider credentials and internal configuration must never be exposed.

---

# 9. API Key Management

Administrative endpoints require an authenticated administrator.

## 9.1 Create API Key

`POST /v1/admin/api-keys`

Request:

```json
{
  "name": "support-assistant-production",
  "project_id": "project_123",
  "scopes": [
    "inference:create",
    "model:read"
  ],
  "expires_at": "2027-01-01T00:00:00Z"
}
```

Response:

HTTP 201:

```json
{
  "id": "key_123",
  "name": "support-assistant-production",
  "key": "lg_live_xxxxxxxxxxxxx",
  "key_prefix": "lg_live_abc",
  "status": "active",
  "created_at": "2026-11-03T12:00:00Z",
  "expires_at": "2027-01-01T00:00:00Z"
}
```

The raw key is returned only during creation.

It must never be returned again by the list or retrieve endpoints.

## 9.2 List API Keys

`GET /v1/admin/api-keys`

Response:

```json
{
  "data": [
    {
      "id": "key_123",
      "name": "support-assistant-production",
      "key_prefix": "lg_live_abc",
      "status": "active",
      "created_at": "2026-11-03T12:00:00Z",
      "last_used_at": "2026-11-03T13:00:00Z"
    }
  ],
  "pagination": {
    "limit": 20,
    "next_cursor": null
  }
}
```

## 9.3 Revoke API Key

`POST /v1/admin/api-keys/{id}/revoke`

Response:

```json
{
  "id": "key_123",
  "status": "revoked",
  "revoked_at": "2026-11-03T14:00:00Z"
}
```

Revocation must take effect across gateway instances within a documented propagation target.

## 9.4 Rotate API Key

`POST /v1/admin/api-keys/{id}/rotate`

Rotation must:

- Generate a new random credential.
- Persist only its hash.
- Return the new credential once.
- Audit the operation.
- Apply an explicit old-key revocation or overlap policy.

---

# 10. Budget API

## 10.1 Create Budget

`POST /v1/admin/budgets`

Request:

```json
{
  "name": "production-monthly-budget",
  "tenant_id": "tenant_123",
  "project_id": "project_123",
  "limit": {
    "amount": "500.00",
    "currency": "USD"
  },
  "period": "monthly",
  "starts_at": "2026-11-01T00:00:00Z",
  "ends_at": "2026-12-01T00:00:00Z"
}
```

## 10.2 Retrieve Budget

`GET /v1/admin/budgets/{id}`

Response:

```json
{
  "id": "budget_123",
  "name": "production-monthly-budget",
  "limit": {
    "amount": "500.00",
    "currency": "USD"
  },
  "spent": {
    "amount": "120.00",
    "currency": "USD"
  },
  "reserved": {
    "amount": "15.00",
    "currency": "USD"
  },
  "available": {
    "amount": "365.00",
    "currency": "USD"
  },
  "status": "active"
}
```

Money values are serialized as decimal strings to avoid floating-point ambiguity.

---

# 11. Usage API

## 11.1 List Usage

`GET /v1/usage`

Supported query parameters:

```text
from
to
project_id
provider_id
model_id
limit
cursor
```

Example:

`GET /v1/usage?from=2026-11-01T00:00:00Z&to=2026-11-03T23:59:59Z&limit=20`

Response:

```json
{
  "data": [
    {
      "execution_id": "exec_123",
      "provider": "example-provider",
      "model": "model-a",
      "input_tokens": 120,
      "output_tokens": 85,
      "total_tokens": 205,
      "estimated_cost": {
        "amount": "0.000420",
        "currency": "USD"
      },
      "actual_cost": {
        "amount": "0.000420",
        "currency": "USD"
      },
      "created_at": "2026-11-03T12:00:00Z"
    }
  ],
  "pagination": {
    "limit": 20,
    "next_cursor": null
  }
}
```

The API must enforce tenant and project ownership regardless of supplied query parameters.

---

# 12. Rate-Limit and Admission Responses

A rejected request returns HTTP 429.

Example:

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 10
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1790000010
```

Response:

```json
{
  "error": {
    "type": "rate_limit_error",
    "code": "rate_limit_exceeded",
    "message": "The request rate limit has been exceeded.",
    "request_id": "req_123",
    "retry_after_seconds": 10
  }
}
```

For concurrency rejection:

```json
{
  "error": {
    "type": "concurrency_error",
    "code": "concurrency_limit_exceeded",
    "message": "The maximum number of concurrent requests has been reached.",
    "request_id": "req_123",
    "retry_after_seconds": 5
  }
}
```

For budget exhaustion:

```json
{
  "error": {
    "type": "budget_error",
    "code": "budget_exceeded",
    "message": "The configured spending limit has been reached.",
    "request_id": "req_123"
  }
}
```

Rate-limit headers may be returned on successful requests too, where appropriate.

The gateway should avoid exposing internal provider limits or another tenant's quota information.

---

# 13. Standard Error Contract

All API errors use a consistent structure.

```json
{
  "error": {
    "type": "authentication_error",
    "code": "invalid_api_key",
    "message": "The supplied API key is invalid.",
    "request_id": "req_123",
    "details": {}
  }
}
```

### Error types

| Type | HTTP Status | Examples |
|---|---:|---|
| validation_error | 400 | Invalid payload |
| authentication_error | 401 | Invalid API key |
| authorization_error | 403 | Insufficient permissions |
| not_found_error | 404 | Unknown resource |
| conflict_error | 409 | Idempotency conflict |
| budget_error | 402 | Budget exhausted |
| rate_limit_error | 429 | Request limit exceeded |
| concurrency_error | 429 | Concurrency capacity exceeded |
| provider_error | 502 | Provider failure |
| provider_timeout | 504 | Provider timeout |
| service_unavailable | 503 | Gateway dependency unavailable |
| internal_error | 500 | Unexpected error |

The API should use stable machine-readable codes. Client applications should not rely on human-readable message text for program logic.

---

# 14. Idempotency Contract

Supported for:

- Chat completion requests.
- Embedding requests.
- Administrative operations that create resources.

Header:

```http
Idempotency-Key: client-generated-unique-value
```

Behavior:

| Scenario | Expected Result |
|---|---|
| First request | Execute normally |
| Same key and same payload, completed | Return stored result |
| Same key and same payload, in progress | Return documented in-progress response |
| Same key and different payload | Return 409 conflict |
| Expired idempotency record | Treat according to retention policy |

Example conflict:

```json
{
  "error": {
    "type": "conflict_error",
    "code": "idempotency_key_reused",
    "message": "The idempotency key was previously used with a different request.",
    "request_id": "req_123"
  }
}
```

The gateway must not claim exactly-once provider execution. If a provider times out after accepting a request, the gateway may not know whether inference was completed.

---

# 15. Pagination

Use cursor-based pagination.

Query:

```text
GET /v1/admin/api-keys?limit=20&cursor=abc123
```

Response:

```json
{
  "data": [],
  "pagination": {
    "limit": 20,
    "next_cursor": "next_cursor_value"
  }
}
```

Rules:

- Default limit: 20.
- Maximum limit: 100.
- Stable ordering.
- Cursor must be opaque.
- Scope and authorization checks apply to every page.

---

# 16. Request Correlation

The gateway should support:

```http
X-Request-ID: client-request-id
Idempotency-Key: unique-request-key
```

If `X-Request-ID` is absent, the gateway generates one.

The gateway should return:

```http
X-Request-ID: generated-request-id
```

A request ID identifies the HTTP request. An execution ID identifies an inference execution. These are separate identifiers.

---

# 17. Administrative Provider Management

Provider credentials must be configured through secret references, not raw credential values in ordinary API payloads.

Example:

`POST /v1/admin/providers`

```json
{
  "name": "primary-openai",
  "provider_type": "openai",
  "base_endpoint": "https://api.openai.com/v1",
  "secret_reference": {
    "secret_name": "llm-gateway/providers/openai/production"
  },
  "region": "global"
}
```

The API stores the secret reference metadata.

It must not accept provider credentials in query strings, logs, or ordinary persisted configuration.

Provider testing must not expose credentials or return sensitive provider response headers.

---

# 18. Health Endpoints

## Liveness

`GET /health/live`

Returns whether the process is alive.

```json
{
  "status": "ok"
}
```

## Readiness

`GET /health/ready`

Returns whether the instance can accept requests.

```json
{
  "status": "ready",
  "dependencies": {
    "postgres": "ok",
    "redis": "ok",
    "event_broker": "ok"
  }
}
```

Readiness must reflect required dependencies and configured failure policies. A provider outage should not necessarily make the entire gateway unready if healthy providers remain available.

---

# 19. OpenAPI Contract

The project must maintain a versioned OpenAPI specification.

Location:

```text
docs/openapi/openapi.yaml
```

Requirements:

- OpenAPI 3.1.
- Reusable schemas.
- Reusable error responses.
- Authentication schemes.
- Request validation constraints.
- Response examples.
- Pagination definitions.
- Streaming behavior documented separately where OpenAPI cannot fully express it.
- CI validation of the OpenAPI document.
- Contract tests against the running API.

The OpenAPI specification is the source of truth for the public HTTP contract.

Rust request DTOs must be tested against the documented schemas.

---

# 20. API Security Requirements

- HTTPS in production.
- No secrets in URL query parameters.
- Authentication before protected resource access.
- Authorization for every administrative operation.
- Tenant isolation for all tenant-owned resources.
- Request body size limits.
- Input validation.
- Rate limiting.
- Abuse protection.
- Secure response headers.
- No sensitive content in default logs.
- Consistent security error responses.
- Audit administrative actions.
- Prevent mass assignment of protected fields.
- Restrict metrics and operational endpoints.

---

# 21. API Contract Definition of Done

- [ ] API versioning is defined.
- [ ] Authentication headers are documented.
- [ ] Administrative authentication is separated from inference authentication.
- [ ] Inference request and response contracts are defined.
- [ ] Streaming behavior is defined.
- [ ] Embedding API is defined.
- [ ] Model discovery endpoints are defined.
- [ ] API key lifecycle endpoints are defined.
- [ ] Tenant and project endpoints are defined.
- [ ] Policy endpoints are defined.
- [ ] Budget endpoints are defined.
- [ ] Usage endpoints are defined.
- [ ] Rate-limit and concurrency responses are defined.
- [ ] Error response structure is standardized.
- [ ] Idempotency semantics are defined.
- [ ] Pagination is standardized.
- [ ] Request and execution correlation is defined.
- [ ] Provider secret references are documented.
- [ ] Health endpoints are defined.
- [ ] OpenAPI contract is planned.
- [ ] Contract testing requirements are defined.

---

# 22. Next Document

The next specification is `docs/security.md`.

It will cover:

- Threat model and attack surfaces.
- Authentication and authorization flows.
- API key hashing and rotation.
- Secret management and credential rotation.
- Tenant isolation.
- RBAC and scopes.
- SSRF and provider endpoint protection.
- Prompt injection and untrusted content boundaries.
- PII handling and data retention.
- Encryption in transit and at rest.
- Audit logging.
- Abuse protection.
- Dependency and supply-chain security.
- Security testing and incident response.
